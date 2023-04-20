use los_rustybandidos::{self, config, inicializar};

#[test]
fn integration_test_config() {
    cargar_config_test();
}

#[test]
fn cargar_config_test() {
    let args: Vec<String> = vec![
        "target".to_string(),
        "src/test_files/test_1.conf".to_string(),
    ];
    let init_result = inicializar(args);
    assert!(init_result.is_ok());

    let nombre_grupo_result = config::get_valor("NOMBRE_GRUPO".to_string());
    assert!(nombre_grupo_result.is_ok());

    let nombre_grupo = nombre_grupo_result.unwrap();
    assert_eq!(nombre_grupo, "Rustybandidos Test".to_string())
}
