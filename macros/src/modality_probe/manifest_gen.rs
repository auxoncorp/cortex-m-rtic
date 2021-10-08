use proc_macro2::TokenStream as TokenStream2;
use quote::ToTokens;
use rtic_syntax::ast::App;
use serde::{Deserialize, Serialize};
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

use crate::modality_probe::id_gen;

const PROBE_CLI: &str = "modality-probe";

pub fn manifest_gen(app: &App) -> TokenStream2 {
    let manifest_dir = if let Some(d) = option_env!("MODALITY_PROJECT_ROOT_DIR") {
        PathBuf::from(d)
    } else {
        env::current_dir().expect("Could not determine the current working directory")
    };

    let generated_src_file = if let Some(d) = option_env!("MODALITY_GENERATED_SOURCE_DIR") {
        PathBuf::from(d)
    } else {
        manifest_dir.join("target")
    }
    .join("modality_component_definitions.rs");
    let component_dir = if let Some(d) = option_env!("MODALITY_COMPONENT_DIR") {
        PathBuf::from(d)
    } else {
        manifest_dir.join("modality-component")
    };

    run_manifest_gen(&component_dir);

    let component_file = Component::component_manifest_path(&component_dir);
    if component_file.exists() {
        let component = Component::from_toml(&component_file);
        let probes_file = Component::probes_manifest_path(&component_dir);
        let mut probes = Probes::from_csv(&probes_file);

        for (task_name, probe) in app.modality_probes() {
            let name = probe.name.to_string().to_uppercase();
            if let Some(pos) = probes.0.iter().position(|p| p.name == name) {
                probes.0[pos].description = probe
                    .description
                    .as_ref()
                    .map(|d| d.value())
                    .unwrap_or_else(|| format!("Probe for task {}", task_name));
            } else {
                let probe_id = ProbeId(id_gen(&probe.name));
                probes.0.push(Probe {
                    component_id: component.id.clone(),
                    id: probe_id,
                    name: probe.name.to_string().to_uppercase(),
                    description: probe
                        .description
                        .as_ref()
                        .map(|d| d.value())
                        .unwrap_or_else(|| format!("Probe for task {}", task_name)),
                    tags: String::from("RTIC;task"),
                    file: String::new(),
                    line: String::new(),
                });
            }
        }

        probes.0.sort_by_key(|p| p.id);
        probes.write_csv(&probes_file);
    }

    run_header_gen(&component_dir, &generated_src_file);

    let generated_src = fs::read_to_string(&generated_src_file)
        .expect("Failed to read generated component definitions");
    syn::parse_file(&generated_src)
        .expect("Failed to parse generated component definitions")
        .into_token_stream()
}

fn run_manifest_gen<P: AsRef<Path>>(component_dir: P) {
    let cli = Path::new(&format!("{}{}", PROBE_CLI, env::consts::EXE_SUFFIX)).to_path_buf();

    // Use the CLI to generate component manifests
    let status = Command::new(&cli)
        .args(&[
            "manifest-gen",
            "--verbose",
            "--lang",
            "rust",
            "--file-extension",
            "rs",
            "--component-name",
            "modality-component",
            "--output-path",
            component_dir.as_ref().to_str().unwrap(),
            "src/main.rs",
        ])
        .status()
        .unwrap();
    assert!(status.success(), "Could not generate component manifests");
}

fn run_header_gen<P: AsRef<Path>>(component_dir: P, generated_src_file: P) {
    let cli = Path::new(&format!("{}{}", PROBE_CLI, env::consts::EXE_SUFFIX)).to_path_buf();

    // Use the CLI to generate Rust definitions from the component manifests
    let status = Command::new(&cli)
        .args(&[
            "header-gen",
            "--lang",
            "rust",
            "--output-path",
            generated_src_file.as_ref().to_str().unwrap(),
            component_dir.as_ref().to_str().unwrap(),
        ])
        .status()
        .unwrap();
    assert!(status.success(), "Could not generate component definitions");
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize)]
struct Component {
    pub name: String,
    pub id: String,
    #[serde(default)]
    pub code_hash: Option<String>,
    #[serde(default)]
    pub instrumentation_hash: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

impl Component {
    pub fn component_manifest_path<P: AsRef<Path>>(component_directory: P) -> PathBuf {
        component_directory.as_ref().join("Component.toml")
    }

    pub fn probes_manifest_path<P: AsRef<Path>>(component_directory: P) -> PathBuf {
        component_directory.as_ref().join("probes.csv")
    }

    pub fn from_toml<P: AsRef<Path>>(path: P) -> Self {
        let content = &fs::read_to_string(path).expect("Can't open component manifest file");
        toml::from_str(content).expect("Can't deserialize component")
    }
}

#[derive(
    Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, Deserialize, Serialize,
)]
struct ProbeId(pub u32);

#[derive(Clone, Debug, PartialEq, PartialOrd, Deserialize, Serialize)]
struct Probe {
    pub component_id: String,
    pub id: ProbeId,
    pub name: String,
    pub description: String,
    pub tags: String,
    pub file: String,
    pub line: String,
}

#[derive(Clone, PartialEq, PartialOrd, Debug)]
struct Probes(Vec<Probe>);

impl Probes {
    pub fn from_csv<P: AsRef<Path>>(path: P) -> Self {
        let probes: Vec<Probe> = if path.as_ref().exists() {
            let mut reader = csv::Reader::from_reader(
                fs::File::open(&path).expect("Can't open probes manifest file"),
            );
            reader
                .deserialize()
                .map(|maybe_probe| maybe_probe.expect("Can't deserialize probe"))
                .map(|mut t: Probe| {
                    t.name = t.name.to_uppercase();
                    t
                })
                .collect()
        } else {
            vec![]
        };

        Probes(probes)
    }

    pub fn write_csv<P: AsRef<Path>>(&self, path: P) {
        let mut writer = csv::Writer::from_writer(
            fs::File::create(&path).expect("Can't create probes manifest file"),
        );
        self.0
            .iter()
            .for_each(|probe| writer.serialize(probe).expect("Can't serialize probe"));
        writer.flush().expect("Can't flush probes writer");
    }
}
