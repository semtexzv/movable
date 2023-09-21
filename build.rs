fn main() -> protokit_build::Result<()> {
    protokit_build::Build::new()
        .compile("movable.proto")?
        .out_dir("gen")
        .generate()
}
