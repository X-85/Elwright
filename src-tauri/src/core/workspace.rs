//! Local-first resource and topic workspace storage.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorkspaceData {
    #[serde(default)]
    pub folders: Vec<Folder>,
    #[serde(default)]
    pub resources: Vec<Resource>,
    #[serde(default)]
    pub topics: Vec<Topic>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Folder {
    pub id: String,
    pub name: String,
    #[serde(rename = "parentId")]
    pub parent_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    pub id: String,
    pub title: String,
    pub kind: String,
    pub value: String,
    #[serde(rename = "folderId")]
    pub folder_id: Option<String>,
    #[serde(default)]
    pub note: String,
    /// app 快捷方式的附加参数。不经 shell 解析，避免把参数当作命令执行。
    #[serde(rename = "launchArgs", default)]
    pub launch_args: Vec<String>,
    /// 内置图标名称，仅作 UI 展示；为空时使用默认应用图标。
    #[serde(default)]
    pub icon: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Topic {
    pub id: String,
    pub title: String,
    pub question: String,
    #[serde(rename = "resourceIds", default)]
    pub resource_ids: Vec<String>,
    #[serde(default)]
    pub report: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

fn now_id(prefix: &str) -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("{}-{:x}", prefix, nanos)
}

pub fn path(root: &Path) -> PathBuf {
    root.join("workspace.json")
}

pub fn load(root: &Path) -> Result<WorkspaceData, String> {
    let file = path(root);
    if !file.is_file() {
        return Ok(WorkspaceData::default());
    }
    let text = fs::read_to_string(&file).map_err(|e| format!("读取工作区失败: {}", e))?;
    serde_json::from_str(&text).map_err(|e| format!("解析工作区失败: {}", e))
}

pub fn save(root: &Path, data: &WorkspaceData) -> Result<(), String> {
    fs::create_dir_all(root).map_err(|e| format!("创建用户目录失败: {}", e))?;
    let file = path(root);
    let text =
        serde_json::to_string_pretty(data).map_err(|e| format!("序列化工作区失败: {}", e))?;

    // Windows 不允许 rename 覆盖已有文件；直接写目标文件以避免后续保存失败。
    #[cfg(windows)]
    {
        fs::write(&file, text).map_err(|e| format!("保存工作区失败: {}", e))
    }

    #[cfg(not(windows))]
    {
        let tmp = file.with_extension("json.tmp");
        fs::write(&tmp, text).map_err(|e| format!("写入工作区失败: {}", e))?;
        fs::rename(&tmp, &file).map_err(|e| format!("保存工作区失败: {}", e))
    }
}

fn depth(data: &WorkspaceData, parent: Option<&str>) -> usize {
    let mut d = 1;
    let mut current = parent;
    while let Some(id) = current {
        if let Some(folder) = data.folders.iter().find(|f| f.id == id) {
            d += 1;
            current = folder.parent_id.as_deref();
        } else {
            break;
        }
    }
    d
}

pub fn create_folder(root: &Path, name: &str, parent_id: Option<String>) -> Result<Folder, String> {
    let mut data = load(root)?;
    let name = name.trim();
    if name.is_empty() {
        return Err("文件夹名称不能为空".into());
    }
    if let Some(ref parent) = parent_id {
        if !data.folders.iter().any(|f| f.id == *parent) {
            return Err("父文件夹不存在".into());
        }
    }
    if depth(&data, parent_id.as_deref()) > 3 {
        return Err("文件夹最多支持三层嵌套".into());
    }
    let folder = Folder {
        id: now_id("folder"),
        name: name.into(),
        parent_id,
    };
    data.folders.push(folder.clone());
    save(root, &data)?;
    Ok(folder)
}

pub fn delete_folder(root: &Path, id: &str) -> Result<(), String> {
    let mut data = load(root)?;
    if !data.folders.iter().any(|f| f.id == id) {
        return Ok(());
    }
    let mut removed = vec![id.to_string()];
    let mut index = 0;
    while index < removed.len() {
        let parent = removed[index].clone();
        for child in data
            .folders
            .iter()
            .filter(|f| f.parent_id.as_deref() == Some(&parent))
        {
            removed.push(child.id.clone());
        }
        index += 1;
    }
    data.folders.retain(|f| !removed.contains(&f.id));
    for resource in &mut data.resources {
        if resource
            .folder_id
            .as_ref()
            .is_some_and(|fid| removed.contains(fid))
        {
            resource.folder_id = None;
        }
    }
    save(root, &data)
}

pub fn create_resource(root: &Path, mut resource: Resource) -> Result<Resource, String> {
    let mut data = load(root)?;
    if resource.title.trim().is_empty() || resource.value.trim().is_empty() {
        return Err("资源标题和内容不能为空".into());
    }
    if let Some(ref folder) = resource.folder_id {
        if !data.folders.iter().any(|f| f.id == *folder) {
            return Err("目标文件夹不存在".into());
        }
    }
    resource.id = now_id("resource");
    resource.title = resource.title.trim().into();
    data.resources.push(resource.clone());
    save(root, &data)?;
    Ok(resource)
}

pub fn delete_resource(root: &Path, id: &str) -> Result<(), String> {
    let mut data = load(root)?;
    data.resources.retain(|r| r.id != id);
    for topic in &mut data.topics {
        topic.resource_ids.retain(|rid| rid != id);
    }
    save(root, &data)
}

/// 启动用户已保存的软件快捷方式。该操作不经过 shell：只执行明确的路径/命令和
/// 分离后的参数，不能利用参数拼接另一条命令。
pub fn launch_app(root: &Path, id: &str) -> Result<(), String> {
    let data = load(root)?;
    let resource = data
        .resources
        .iter()
        .find(|resource| resource.id == id)
        .ok_or_else(|| "快捷方式不存在".to_string())?;
    if resource.kind != "app" {
        return Err("该资源不是软件快捷方式".into());
    }
    if resource.value.trim().is_empty() {
        return Err("软件路径或命令不能为空".into());
    }

    #[cfg(target_os = "macos")]
    let mut command = {
        if resource.value.ends_with(".app") {
            let mut open = Command::new("open");
            open.arg(&resource.value);
            if !resource.launch_args.is_empty() {
                open.arg("--args").args(&resource.launch_args);
            }
            open
        } else {
            let mut direct = Command::new(&resource.value);
            direct.args(&resource.launch_args);
            direct
        }
    };
    #[cfg(not(target_os = "macos"))]
    let mut command = {
        let mut direct = Command::new(&resource.value);
        direct.args(&resource.launch_args);
        direct
    };

    command
        .spawn()
        .map_err(|e| format!("启动「{}」失败: {}", resource.title, e))?;
    Ok(())
}

pub fn create_topic(root: &Path, title: &str, question: &str) -> Result<Topic, String> {
    let mut data = load(root)?;
    if title.trim().is_empty() {
        return Err("课题名称不能为空".into());
    }
    let topic = Topic {
        id: now_id("topic"),
        title: title.trim().into(),
        question: question.trim().into(),
        resource_ids: Vec::new(),
        report: String::new(),
        updated_at: now_id("t"),
    };
    data.topics.push(topic.clone());
    save(root, &data)?;
    Ok(topic)
}

pub fn update_topic(root: &Path, topic: Topic) -> Result<(), String> {
    let mut data = load(root)?;
    let existing = data
        .topics
        .iter_mut()
        .find(|t| t.id == topic.id)
        .ok_or_else(|| "课题不存在".to_string())?;
    *existing = topic;
    save(root, &data)
}

pub fn delete_topic(root: &Path, id: &str) -> Result<(), String> {
    let mut data = load(root)?;
    data.topics.retain(|topic| topic.id != id);
    save(root, &data)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_root() -> PathBuf {
        std::env::temp_dir().join(format!("elwright-workspace-{}", now_id("test")))
    }

    #[test]
    fn folder_depth_is_limited_to_three() {
        let root = temp_root();
        let a = create_folder(&root, "一", None).unwrap();
        let b = create_folder(&root, "二", Some(a.id)).unwrap();
        let c = create_folder(&root, "三", Some(b.id)).unwrap();
        assert!(create_folder(&root, "四", Some(c.id)).is_err());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn deleting_folder_keeps_resources_but_unlinks_them() {
        let root = temp_root();
        let folder = create_folder(&root, "资料", None).unwrap();
        let resource = create_resource(
            &root,
            Resource {
                id: String::new(),
                title: "文档".into(),
                kind: "url".into(),
                value: "https://example.com".into(),
                folder_id: Some(folder.id.clone()),
                note: String::new(),
                launch_args: Vec::new(),
                icon: String::new(),
            },
        )
        .unwrap();
        delete_folder(&root, &folder.id).unwrap();
        let data = load(&root).unwrap();
        assert!(data.folders.is_empty());
        assert_eq!(data.resources[0].id, resource.id);
        assert!(data.resources[0].folder_id.is_none());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn serializes_frontend_field_names() {
        let folder = Folder {
            id: "f".into(),
            name: "资料".into(),
            parent_id: None,
        };
        let value = serde_json::to_value(folder).unwrap();
        assert!(value.get("parentId").is_some());
        assert!(value.get("parent_id").is_none());
    }

    #[test]
    fn only_app_resources_can_be_launched() {
        let root = temp_root();
        let resource = create_resource(
            &root,
            Resource {
                id: String::new(),
                title: "网页".into(),
                kind: "url".into(),
                value: "https://example.com".into(),
                folder_id: None,
                note: String::new(),
                launch_args: Vec::new(),
                icon: String::new(),
            },
        )
        .unwrap();
        assert!(launch_app(&root, &resource.id)
            .unwrap_err()
            .contains("不是软件"));
        let _ = fs::remove_dir_all(root);
    }
}
