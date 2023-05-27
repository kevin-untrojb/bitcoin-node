# 23C1-Los-Rustybandidos
Repo for Rust Taller De Programacion 1 FIUBA  
  

## Equipo Los Rustybandidos
Schmidt Agustina  
Spacek Daniel  
Fraccaro Agustina  
Untrojb Kevin  
  
  
## Presentación de la entrega intermedia  
  
Slides:  https://docs.google.com/presentation/d/1IIWP1ySUBLSOf8tgvPSMC4lSymeUIUKowZAmug60pnU/edit?usp=sharing  



## GTK  
  
Sigue los siguientes pasos para instalar la versión dev del GTK:

* Instalar la biblioteca GTK 3. En Ubuntu debes ejecutar:  
` sudo apt-get install libgtk-3-dev `
* Para incluir la referencia en el proyecto, en caso de que no exista en el Cargo.toml, en la terminal de la carpeta del proyecto:  
`cargo add gtk`  
* Para ejecutar la aplicación:  
`cargo run --bin {application_name}`  
* En caso de que al ejecutar la aplicación, muestre un error de :  
    - version `GLIBCXX_3.4.29' not found  

    Debe limpiar la variable GTK_PATH desde la consola del IDE  
    `unset GTK_PATH`  

... y magia!  
