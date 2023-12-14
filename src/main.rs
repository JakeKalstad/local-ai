use serde_derive::Deserialize;
use serde_derive::Serialize;
use std::env;
use std::fs;
use std::fs::File;
use std::io::Seek;
use std::slice::Chunks;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Models {
    pub available: Vec<Model>,
}

impl Models {
    fn list(&self) {
        println!("Models");
        println!("======================");
        for m in &self.available {
            println!("{}", m.name);
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Model {
    pub name: String,
    pub version: String,
    pub url: String,
    #[serde(rename = "server-url")]
    pub server_url: String,
    pub parameters: Vec<Parameter>,
}

enum ModelType {
    CLI,
    Server,
}
impl Model {
    fn params(&self) {
        println!("{} parameters", self.name);
        println!("======================");
        for p in &self.parameters {
            println!("{} - {}", p.switch, p.explanation);
        }
    }

    fn run(&self, params: Chunks<'_, String>) -> Vec<u8> {
        let (_, fname) = self.url.rsplit_once('/').unwrap();
        // ./mistral-7b-instruct-v0.1-Q4_K_M-main.llamafile --temp 0.7 -r '\n' -p '### Instruction: Write a story about llamas\n### Response:\n'
        let mut param_string = String::default();
        for chunk in params {
            let switch = chunk.first().expect("Switch exists");
            let value = chunk.last().expect("value exists");
            if switch == "temp" {
                param_string = format!("{param_string} --temp {value}");
                continue;
            }
            param_string = format!("{param_string} -{switch} \"{value}\"")
        }
        if self.name == "Llava" {
            param_string = format!("{param_string} --silent-prompt 2>/dev/null")
        }
        println!("./{fname} {param_string}");
        run_command(&format!("./{fname} {param_string}"))
    }

    fn download(&self, model_type: ModelType) {
        let url = match model_type {
            ModelType::CLI => &self.url,
            ModelType::Server => &self.server_url,
        };

        run_command(&format!("wget {}", url));
        let (_, fname) = url.rsplit_once('/').unwrap();
        run_command(&format!("chmod +x {fname}"));

        let mut f = File::options()
            .read(true)
            .write(true)
            .open("cached.json")
            .unwrap();
        let mut cached_models: Models = serde_json::from_reader(&f).unwrap_or_default();
        cached_models.available.push(self.clone());
        let _ = f.seek(std::io::SeekFrom::Start(0)).unwrap();
        serde_json::to_writer(f, &cached_models).unwrap();
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Parameter {
    pub switch: String,
    pub explanation: String,
}

fn run_command(cmd: &str) -> Vec<u8> {
    use std::process::Command;
    let output = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/C", cmd])
            .output()
            .expect("failed to execute process")
    } else {
        Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .output()
            .expect("failed to execute process")
    };
    output.stdout
}

fn test() {
    run_command("wget https://huggingface.co/jartine/llava-v1.5-7B-GGUF/resolve/main/llava-v1.5-7b-q4-main.llamafile");
    run_command("chmod +x llava-v1.5-7b-q4-main.llamafile");
    let output = String::from_utf8(run_command("./llava-v1.5-7b-q4-main.llamafile --version"))
        .expect("valid stringable output");
    println!("{output}")
}

fn help() {
    println!("Welcome to LocalAI - A tool to help make local AI painless");
    println!("==========================================================");
    println!("test: downloads and runs the llava model with an arbitrary test prompt");
    println!("list: lists all models recognized by local-ai");
    println!("list all: lists all models currently downloaded and available to run and serve");
    println!("{{model-name}} params: explains the relevant parameters and deﬁnes how to pass them to the speciﬁed model");
    println!("{{model-name}} {{relevant}} {{model}} {{parameters}}: runs the model using the available parameters that are available to the speciﬁed model");
    println!(
        "{{model-name}} serve: creates a listening service to accept model parameter requests"
    );
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() == 1 {
        println!("No arguments supplied, lai help to display possible arguments");
        return;
    }
    let bin_name = args.first().expect("bin argument must exist");
    let base_arg = args.get(1).expect("first argument must exist");

    let config = fs::read_to_string("models.json").expect("configuration not found");
    let models: Models = serde_json::from_str(&config).expect("configuration is malformed");

    let cached_blob = fs::read_to_string("cached.json").expect("configuration not found");
    let cached_models: Models = serde_json::from_str(&cached_blob).unwrap_or_default();

    let mut model_names = vec![];
    for m in &models.available {
        model_names.push(m.name.as_str());
    }

    let mut cached_model_names = vec![];
    for m in &cached_models.available {
        cached_model_names.push(m.name.as_str());
    }

    if model_names.contains(&base_arg.as_str()) {
        let mut model = Model::default();
        let mut param_names = vec![];
        for m in &models.available {
            if m.name.as_str() == base_arg {
                model = m.clone();
                for p in m.clone().parameters {
                    param_names.push(p.switch);
                }
            }
        }
        if args.len() > 2 {
            let model_arg = args.get(2).expect("model argument must exist");
            match model_arg.as_str() {
                "run" => {
                    let model_args = args.split_at(3).1.chunks(2).clone();
                    for chunk in model_args.clone() {
                        let switch = chunk.first().expect("param switch exists");
                        if !param_names.contains(switch) {
                            println!("{switch} invalid model parameter, to view valid parameters use ./lai {base_arg} params");
                            return;
                        }
                        let value = chunk.last().expect("param value exists");
                        if value.len() == 0 {
                            println!("{switch} invalid model parameter value, to view valid parameters use ./lai {base_arg} params");
                            return;
                        }
                        if !cached_model_names.contains(&base_arg.as_str()) {
                            model.download(ModelType::CLI);
                            return;
                        }
                    }
                    let output = model.run(model_args.clone());
                    println!("{}", String::from_utf8(output).expect("Output exists"));
                }
                "params" => model.params(),
                "serve" => {
                    let model_args = args.split_at(3).1.chunks(2).clone();
                    for chunk in model_args.clone() {
                        let switch = chunk.first().expect("param switch exists");
                        if !param_names.contains(switch) {
                            println!("{switch} invalid model parameter, to view valid parameters use ./lai {base_arg} params");
                            return;
                        }
                        let value = chunk.last().expect("param value exists");
                        if value.len() == 0 {
                            println!("{switch} invalid model parameter value, to view valid parameters use ./lai {base_arg} params");
                            return;
                        }
                        if !cached_model_names.contains(&base_arg.as_str()) {
                            model.download(ModelType::Server);
                            return;
                        }
                    }
                    let output = model.run(model_args.clone());
                    println!("{}", String::from_utf8(output).expect("Output exists"));
                }
                _ => {
                    println!("Invalid Model argument, ie: ./lai {base_arg} {{params|run|serve}}")
                }
            }
        } else {
            println!("Model argument must exist, ie: ./lai {base_arg} {{params|run|serve}}")
        }
        return;
    }

    match base_arg.as_str() {
        "help" => help(),
        "test" => test(),
        "list" => {
            if args.len() > 2 {
                cached_models.list()
            } else {
                models.list()
            }
        }
        _ => {
            println!("No valid argument supplied - see '{bin_name} help' or readme.")
        }
    };
}
