#[derive(Debug, Clone)]
pub struct Scene {
    pub index: usize,
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct Plan {
    pub scenes: Vec<Scene>,
}

pub fn build_plan(text: &str, scene_chars: usize) -> Plan {
    let clean = text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n");

    if clean.is_empty() {
        return Plan { scenes: Vec::new() };
    }

    let mut scenes = Vec::new();
    let mut current = String::new();
    let target = scene_chars.max(80);

    for paragraph in clean.split('\n') {
        if current.chars().count() + paragraph.chars().count() + 1 > target && !current.is_empty() {
            scenes.push(current.clone());
            current.clear();
        }

        if !current.is_empty() {
            current.push('\n');
        }
        current.push_str(paragraph);
    }

    if !current.is_empty() {
        scenes.push(current);
    }

    let scenes = scenes
        .into_iter()
        .enumerate()
        .map(|(idx, text)| Scene {
            index: idx + 1,
            text,
        })
        .collect();

    Plan { scenes }
}
