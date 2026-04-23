use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{GrinddError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BuildInstruction {
    From(String),
    Run(String),
    Copy { src: String, dst: String },
    Cmd(Vec<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildPlan {
    pub instructions: Vec<BuildInstruction>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct BuildCache {
    pub layers: HashMap<String, String>,
}

pub fn parse_build_file(path: &Path) -> Result<BuildPlan> {
    let content = fs::read_to_string(path)?;
    let mut instructions = Vec::new();

    for raw_line in content.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(v) = line.strip_prefix("FROM ") {
            instructions.push(BuildInstruction::From(v.trim().to_string()));
            continue;
        }
        if let Some(v) = line.strip_prefix("RUN ") {
            instructions.push(BuildInstruction::Run(v.trim().to_string()));
            continue;
        }
        if let Some(v) = line.strip_prefix("COPY ") {
            let parts: Vec<_> = v.split_whitespace().collect();
            if parts.len() != 2 {
                return Err(GrinddError::Build(format!("invalid COPY line: {line}")));
            }
            instructions.push(BuildInstruction::Copy {
                src: parts[0].to_string(),
                dst: parts[1].to_string(),
            });
            continue;
        }
        if let Some(v) = line.strip_prefix("CMD ") {
            instructions.push(BuildInstruction::Cmd(v.split_whitespace().map(ToString::to_string).collect()));
            continue;
        }
        return Err(GrinddError::Build(format!("unknown instruction: {line}")));
    }

    Ok(BuildPlan { instructions })
}

pub fn execute_build(plan: &BuildPlan, context_dir: &Path, cache_path: &Path) -> Result<Vec<String>> {
    let mut cache = load_cache(cache_path)?;
    let mut built_layers = Vec::new();

    for instruction in &plan.instructions {
        let key = instruction_key(instruction, context_dir)?;
        if let Some(layer) = cache.layers.get(&key) {
            built_layers.push(layer.clone());
            continue;
        }
        let layer_id = format!("layer-{}", built_layers.len());
        cache.layers.insert(key, layer_id.clone());
        built_layers.push(layer_id);
    }

    let payload = serde_json::to_vec_pretty(&cache)
        .map_err(|e| GrinddError::Build(format!("cache serialize failed: {e}")))?;
    fs::write(cache_path, payload)?;

    Ok(built_layers)
}

fn instruction_key(instruction: &BuildInstruction, context_dir: &Path) -> Result<String> {
    let text = match instruction {
        BuildInstruction::From(x) => format!("FROM:{x}"),
        BuildInstruction::Run(x) => format!("RUN:{x}"),
        BuildInstruction::Copy { src, dst } => {
            let source = context_dir.join(src);
            let digest = if source.exists() {
                let bytes = fs::read(&source)?;
                let mut hasher = Sha256::new();
                hasher.update(bytes);
                hex::encode(hasher.finalize())
            } else {
                "missing".to_string()
            };
            format!("COPY:{src}:{dst}:{digest}")
        }
        BuildInstruction::Cmd(args) => format!("CMD:{}", args.join(" ")),
    };

    Ok(text)
}

fn load_cache(path: &Path) -> Result<BuildCache> {
    if !path.exists() {
        return Ok(BuildCache::default());
    }
    let data = fs::read(path)?;
    serde_json::from_slice(&data).map_err(|e| GrinddError::Build(format!("cache parse failed: {e}")))
}

pub fn default_cache_path(state_root: &Path) -> PathBuf {
    state_root.join("build-cache.json")
}
