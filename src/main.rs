mod config;

fn main() {
    let config_filename = "src/nodo.conf".to_string();
    let _config_file = config::init_config(config_filename);

    println!("Hello, Bitcoin!");
}
