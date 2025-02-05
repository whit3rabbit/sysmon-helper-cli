#[cfg(test)]
mod sysmon_modular_tests {
    use std::path::PathBuf;
    use tempfile::tempdir;
    use sysmon_json::{
        convert_file,
        merge_configs,
    };
    use sysmon_validator::validate_config;
    use walkdir::WalkDir;

    fn get_fixture_path() -> PathBuf {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("tests");
        path.push("fixtures");
        path.push("sysmon-modular");
        path
    }

    #[test]
    fn test_validate_default_config() {
        let fixture_path = get_fixture_path();
        let config_path = fixture_path.join("sysmonconfig.xml");
        assert!(config_path.exists(), "Default sysmon config not found");
        
        let result = validate_config(&config_path);
        assert!(result.is_ok(), "Default config validation failed: {:?}", result.err());
    }

    #[test]
    fn test_convert_default_config_to_json() {
        let fixture_path = get_fixture_path();
        let config_path = fixture_path.join("sysmonconfig.xml");
        let temp_dir = tempdir().unwrap();
        let output_path = temp_dir.path().join("sysmonconfig.json");

        let result = convert_file(&config_path, &output_path);
        assert!(result.is_ok(), "Failed to convert config to JSON: {:?}", result.err());
        assert!(output_path.exists(), "JSON output file not created");
    }

    #[test]
    fn test_merge_process_creation_configs() {
        let fixture_path = get_fixture_path();
        let process_path = fixture_path.join("1_process_creation");
        let temp_dir = tempdir().unwrap();
        let output_path = temp_dir.path().join("merged_process.xml");

        let result = merge_configs(&process_path, &output_path, true);
        assert!(result.is_ok(), "Failed to merge process creation configs: {:?}", result.err());
        
        // Validate merged config
        let validation = validate_config(&output_path);
        assert!(validation.is_ok(), "Merged config validation failed: {:?}", validation.err());
    }

    #[test]
    fn test_merge_all_event_configs() {
        let fixture_path = get_fixture_path();
        let temp_dir = tempdir().unwrap();
        let output_path = temp_dir.path().join("merged_all.xml");

        // Get all numbered directories (event type configs)
        let event_dirs: Vec<PathBuf> = WalkDir::new(&fixture_path)
            .min_depth(1)
            .max_depth(1)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.chars().next().unwrap().is_numeric())
                    .unwrap_or(false)
            })
            .map(|e| e.path().to_owned())
            .collect();

        assert!(!event_dirs.is_empty(), "No event directories found");

        // Merge all event configs
        let result = merge_configs(&fixture_path, &output_path, true);
        assert!(result.is_ok(), "Failed to merge all configs: {:?}", result.err());
        
        // Validate merged config
        let validation = validate_config(&output_path);
        assert!(validation.is_ok(), "Full merged config validation failed: {:?}", validation.err());
    }

    #[test]
    fn test_convert_all_configs_to_json() {
        let fixture_path = get_fixture_path();
        let temp_dir = tempdir().unwrap();
        
        // Find all XML files
        let xml_files: Vec<PathBuf> = WalkDir::new(&fixture_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| ext.eq_ignore_ascii_case("xml"))
                    .unwrap_or(false)
            })
            .map(|e| e.path().to_owned())
            .collect();

        assert!(!xml_files.is_empty(), "No XML files found");

        for xml_path in xml_files {
            let file_name = xml_path.file_name().unwrap();
            let json_path = temp_dir.path().join(file_name).with_extension("json");
            
            let result = convert_file(&xml_path, &json_path);
            assert!(result.is_ok(), "Failed to convert {:?} to JSON: {:?}", xml_path, result.err());
            assert!(json_path.exists(), "JSON output file not created for {:?}", xml_path);
        }
    }

    #[test]
    fn test_validate_all_configs() {
        let fixture_path = get_fixture_path();
        
        // Known invalid configs that we want to keep for reference
        let known_invalid = [
            "11_file_create/include_cve_2021_40444.xml", // all event filtering rules (like FileCreate, ProcessCreate, etc.) must be wrapped in an <EventFiltering> element
            "sysmonconfig-research.xml", // Each event type (like ProcessCreate, FileCreateTime, etc.) must be wrapped in a RuleGroup element
            "templates/sysmon_template.xml", // Requires at least one RuleGroup element inside EventFiltering
            ];
        
        let xml_files: Vec<PathBuf> = WalkDir::new(&fixture_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| ext.eq_ignore_ascii_case("xml"))
                    .unwrap_or(false)
            })
            .map(|e| e.path().to_owned())
            .collect();
    
        assert!(!xml_files.is_empty(), "No XML files found");
    
        for xml_path in xml_files {
            let relative_path = xml_path.strip_prefix(&fixture_path)
                .unwrap()
                .to_str()
                .unwrap()
                .replace('\\', "/");
                
            let result = validate_config(&xml_path);
            
            if known_invalid.contains(&relative_path.as_str()) {
                assert!(result.is_err(), "Expected config to be invalid: {:?}", xml_path);
            } else {
                if let Err(ref e) = result {
                    println!("Validation error for {:?}: {}", xml_path, e);
                    if let Ok(content) = std::fs::read_to_string(&xml_path) {
                        println!("File content:\n{}", content);
                    }
                }
                assert!(result.is_ok(), "Config validation failed for {:?}: {:?}", xml_path, result.err());
            }
        }
    }
}