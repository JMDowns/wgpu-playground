pub fn translate_template_filepath_to_artifact_filepath(path: &str) -> String {
    let prefix = "build/shaders/";
    let suffix = ".template.wgsl";
    let new_prefix = "build/shaders/artifacts/";
    let new_suffix = ".wgsl";
   
    let middle = &path[prefix.len()..path.len() - suffix.len()];
    format!("{}{}{}", new_prefix, middle, new_suffix)
}