fn main() {
    let mut config = prost_build::Config::default();
    config.out_dir("src/proto");
    config
        .compile_protos(&["proto/helloworld.proto"], &["proto/"])
        .unwrap();
    // prost_build::compile_protos(&["proto/status.proto"], &["proto/"]).unwrap();
}