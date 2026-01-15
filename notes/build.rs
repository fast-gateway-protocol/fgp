fn main() {
    prost_build::compile_protos(&["proto/notestore.proto"], &["proto/"]).unwrap();
}
