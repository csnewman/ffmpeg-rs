use bindgen::callbacks::{EnumVariantValue, MacroParsingBehavior, ParseCallbacks};
use convert_case::{Case, Casing};
use flate2::read::GzDecoder;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::collections::HashSet;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, Cursor, Read};
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};
use tar::{Archive, Entry, EntryType};

const DOWNLOAD_URL: &'static str =
    "https://github.com/FFmpeg/FFmpeg/archive/refs/heads/release/5.0.tar.gz";

const ENABLED_LIBRARIES: &'static [&'static str] = &[
    // Core
    "avcodec",
    "avdevice",
    "avfilter",
    "avformat",
    "avutil",
    "postproc",
    "swresample",
    "swscale",
    "openssl",
    // Filters
    "fontconfig",
    "frei0r",
    "ladspa",
    "libass",
    "libfreetype",
    "libfribidi",
    // Encoders & Decoders
    "libgsm",
    "libmp3lame",
    "libopencore-amrnb",
    "libopencore-amrwb",
    "libopenh264",
    "libopenjpeg",
    "libopus",
    "libshine",
    "libsnappy",
    "libspeex",
    "libtheora",
    "libtwolame",
    "libvo-amrwbenc",
    "libvorbis",
    "libvpx",
    "libwebp",
    "libx264",
    "libx265",
    "libxvid",
    "libdrm",
];

const ENABLED_HEADERS: &'static [&'static str] = &[
    "libavutil/macros.h",
    "libavutil/error.h",
    "libavcodec/avcodec.h",
    "libavformat/avformat.h",
];

lazy_static! {
    static ref CUSTOM_PREFIX: HashMap<String, &'static str> = {
        let mut m = HashMap::new();
        m.insert("AVPixelFormat".to_string(), "AVPIXFMT");
        m
    };
}

#[derive(Debug)]
struct Callbacks;

impl ParseCallbacks for Callbacks {
    fn will_parse_macro(&self, name: &str) -> MacroParsingBehavior {
        match name {
            "FP_INFINITE" => MacroParsingBehavior::Ignore,
            "FP_NAN" => MacroParsingBehavior::Ignore,
            "FP_NORMAL" => MacroParsingBehavior::Ignore,
            "FP_SUBNORMAL" => MacroParsingBehavior::Ignore,
            "FP_ZERO" => MacroParsingBehavior::Ignore,
            _ => MacroParsingBehavior::Default,
        }
    }

    fn enum_variant_name(
        &self,
        enum_name: Option<&str>,
        original_variant_name: &str,
        _variant_value: EnumVariantValue,
    ) -> Option<String> {
        let enum_name = enum_name.unwrap().to_string();
        let mut enum_name: Vec<String> = enum_name.split(" ").map(|x| x.to_string()).collect();
        let enum_name = if enum_name.len() == 1 || enum_name.len() == 2 {
            enum_name.pop().unwrap()
        } else {
            return None;
        };

        let mut chars = match CUSTOM_PREFIX.get(&enum_name) {
            None => enum_name.chars().peekable(),
            Some(value) => value.chars().peekable(),
        };
        let mut new_name = String::new();

        let mut copying = false;
        for c in original_variant_name.chars() {
            if copying {
                new_name.push(c);
                continue;
            } else if c == '_' {
                continue;
            }

            if let Some(next_char) = chars.peek() {
                if next_char.to_ascii_lowercase() == c.to_ascii_lowercase() {
                    chars.next();
                    continue;
                }
            }

            copying = true;
            new_name.push(c);
        }

        let mut new_name = new_name
            .from_case(Case::UpperSnake)
            .to_case(Case::UpperCamel);

        // Ensure name begins with a valid character
        if !new_name.chars().next().unwrap().is_alphabetic() {
            new_name.insert(0, '_');
        }

        Some(new_name)
    }
}

fn main() {
    // Build if missing
    let lib = Path::new::<str>("prefix/lib/libavcodec.a").to_path_buf();
    if !lib.exists() {
        build_ffmpeg();
    }

    //
    let prefix = Path::new("prefix").to_path_buf();
    println!(
        "cargo:rustc-link-search=native={}",
        prefix.join("lib").to_string_lossy()
    );

    println!("cargo:rustc-link-lib=static=avcodec");
    println!("cargo:rustc-link-lib=static=avdevice");
    println!("cargo:rustc-link-lib=static=avfilter");
    println!("cargo:rustc-link-lib=static=avformat");
    println!("cargo:rustc-link-lib=static=avutil");
    println!("cargo:rustc-link-lib=static=postproc");
    println!("cargo:rustc-link-lib=static=swresample");
    println!("cargo:rustc-link-lib=static=swscale");

    {
        let config_mak = Path::new::<str>("build/ffbuild/config.mak").to_path_buf();
        let file = File::open(config_mak).unwrap();
        let lines = BufReader::new(file).lines();

        let mut dynlibs = HashSet::new();
        for line in lines {
            let line = line.unwrap();
            if line.starts_with("EXTRALIBS") {
                let linker_args = line.split('=').last().unwrap().split(' ');
                for arg in linker_args {
                    if arg.starts_with("-l") {
                        let lib = &arg[2..];
                        dynlibs.insert(lib.to_string());
                    }
                }
            }
        }

        for lib in &dynlibs {
            println!("cargo:rustc-link-lib={}", lib);
        }
    }

    let mut builder = bindgen::Builder::default();

    for header in ENABLED_HEADERS {
        builder = builder.header(format!("prefix/include/{}", *header));
    }

    let bindings = builder
        .clang_args([format!("-I{}", prefix.join("include").to_string_lossy())])
        .parse_callbacks(Box::new(Callbacks))
        .rustified_enum("*")
        .generate()
        .expect("Unable to generate bindings")
        .to_string();

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    std::fs::write(out_dir.join("bindings.rs"), &bindings).expect("Couldn't write bindings!");
    // std::fs::write("bindings.rs", &bindings)
    //     .expect("Couldn't write bindings!");
}

fn build_ffmpeg() {
    std::fs::create_dir_all("cache").unwrap();

    // Download file
    let src_archive = Path::new::<str>("cache/ffmpeg-5.0.tar.gz").to_path_buf();
    if !src_archive.exists() {
        println!("Downloading {}", DOWNLOAD_URL);
        let resp = reqwest::blocking::get(DOWNLOAD_URL).expect("Failed to fetch");
        let mut file = File::create(&src_archive).expect("Failed to create file");
        let mut content = Cursor::new(resp.bytes().expect("Failed to get bytes"));
        std::io::copy(&mut content, &mut file).expect("Failed to write file");
    }

    let build_dir = Path::new::<str>("build").to_path_buf();
    if build_dir.exists() {
        std::fs::remove_dir_all(&build_dir).expect("Failed to delete old build");
    }
    std::fs::create_dir_all(&build_dir).unwrap();

    let prefix = Path::new("prefix").to_path_buf();
    std::fs::create_dir_all(&prefix).unwrap();
    let prefix = prefix.canonicalize().unwrap();

    // Extract archive
    {
        let tar_gz = File::open(&src_archive).unwrap();
        let tar = GzDecoder::new(tar_gz);
        let mut archive = Archive::new(tar);
        extract_archive(&mut archive, &build_dir);
    }

    configure_ffmpeg(&build_dir, &prefix);

    {
        let mut cmd = Command::new("make");
        cmd.current_dir(&build_dir);
        println!("Running {:?}", cmd);

        let output = cmd
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output()
            .expect("failed to execute process");
        println!("status: {}", output.status);
        assert!(output.status.success());
    }

    {
        let mut cmd = Command::new("make");
        cmd.current_dir(&build_dir);
        cmd.arg("install");
        println!("Running {:?}", cmd);

        let output = cmd
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output()
            .expect("failed to execute process");
        println!("status: {}", output.status);
        assert!(output.status.success());
    }
}

fn configure_ffmpeg<PP: AsRef<Path>, BP: AsRef<Path>>(build_dir: PP, prefix: BP) {
    let build_dir = build_dir.as_ref();
    let prefix = prefix.as_ref().to_path_buf();

    let configure_path = build_dir.join("configure").canonicalize().unwrap();
    assert!(configure_path.exists());
    let mut cmd = Command::new(configure_path);
    cmd.current_dir(&build_dir);
    cmd.arg(format!("--prefix={}", prefix.to_string_lossy()));
    cmd.arg("--disable-autodetect");
    cmd.arg("--disable-programs");
    cmd.arg("--enable-static");
    cmd.arg("--disable-shared");
    cmd.arg("--disable-debug");
    cmd.arg("--enable-stripping");

    // Version
    cmd.arg("--enable-gpl");
    cmd.arg("--enable-version3");

    for library in ENABLED_LIBRARIES {
        cmd.arg(format!("--enable-{}", library));
    }

    println!("Running {:?}", cmd);
    let output = cmd
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .expect("failed to execute process");
    println!("status: {}", output.status);
    assert!(output.status.success());
}

fn extract_archive<P: AsRef<Path>, R: Read>(archive: &mut Archive<R>, dest: P) {
    let dest = dest.as_ref();

    let mut directories = Vec::new();
    for entry in archive.entries().unwrap() {
        let mut file = entry.unwrap();
        if file.header().entry_type() == EntryType::Directory {
            directories.push(file);
        } else {
            extract_file(&mut file, dest);
        }
    }
    for mut dir in directories {
        extract_file(&mut dir, dest);
    }
}

fn extract_file<R: Read, P: AsRef<Path>>(file: &mut Entry<R>, dst: P) {
    let dst = dst.as_ref();
    let mut file_dst = dst.to_path_buf();
    {
        let mut first = true;
        let path = file.path().unwrap();
        for part in path.components() {
            match part {
                Component::Prefix(..) | Component::RootDir | Component::CurDir => continue,
                Component::ParentDir => return,
                Component::Normal(part) => {
                    if first {
                        first = false;
                    } else {
                        file_dst.push(part)
                    }
                }
            }
        }
    }

    // Skip cases where only slashes or '.' parts were seen, because
    // this is effectively an empty filename.
    if *dst == *file_dst {
        return;
    }

    // Skip entries without a parent (i.e. outside of FS root)
    let parent = match file_dst.parent() {
        Some(p) => p,
        None => return,
    };

    ensure_dir_created(&dst, parent);
    validate_inside_dst(&dst, parent);

    file.unpack(&file_dst).expect("Expected to unwrap file");
}

fn ensure_dir_created(dst: &Path, dir: &Path) {
    let mut ancestor = dir;
    let mut dirs_to_create = Vec::new();
    while ancestor.symlink_metadata().is_err() {
        dirs_to_create.push(ancestor);
        if let Some(parent) = ancestor.parent() {
            ancestor = parent;
        } else {
            break;
        }
    }
    for ancestor in dirs_to_create.into_iter().rev() {
        if let Some(parent) = ancestor.parent() {
            validate_inside_dst(dst, parent);
        }
        std::fs::create_dir_all(ancestor).unwrap();
    }
}

fn validate_inside_dst(dst: &Path, file_dst: &Path) {
    // Abort if target (canonical) parent is outside of `dst`
    let canon_parent = file_dst.canonicalize().unwrap();
    let canon_target = dst.canonicalize().unwrap();
    if !canon_parent.starts_with(&canon_target) {
        panic!(
            "trying to unpack outside of destination path: {}",
            canon_target.display()
        );
    }
}
