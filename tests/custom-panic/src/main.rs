use human_panic::setup_panic;
fn main() {
    setup_panic!(Metadata {
        name: env!("CARGO_PKG_NAME").into(),
        version: env!("CARGO_PKG_VERSION").into(),
        authors: "My Company Support <support@mycompany.com".into(),
        homepage: "support.mycompany.com".into(),
        path_to_save_log_to: match dirs::config_dir(){
            Some(system_config_dir) =>{
                system_config_dir
                        .join("test")
            },
        _=>{
            std::env::temp_dir()
        }
        }
    });
    println!("{:?}",std::env::temp_dir());
    println!("A normal log message");
    panic!("OMG EVERYTHING IS ON FIRE!!!");
}
