fn main() {
    rsvpp_apigen::Generator::new(
        "./vpp_api",
        "/usr/share/vpp/api",
        "/usr/include/vnet/api_errno.h",
    )
    .unwrap()
    .gen()
    .unwrap();
}
