    use std::fs::File;
    use std::io::{self, Read};
    use std::path::Path;
    use csv::ReaderBuilder;
    use serde_json::Value;
    use serde::{Deserialize, Serialize};
    use crate::utils::linked_list::IdentityNode;

    /// Identity structure for gold set records
    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct GoldSetIdentity {
        pub first_name: String,
        pub last_name: String,
        pub father_name: String,
        pub grandfather_name: String,
        pub mother_last_name: String,
        pub mother_name: String,
        pub dob: Option<(u32, u32, u32)>,
        pub sex: u8,
        pub place_of_birth: String,
    }

    /// Represents a record in the gold set file
    #[derive(Debug)]
    pub struct GoldSetRecord {
        pub input_id: String,
        pub candidate_id: String,
        pub is_match: bool,
    }

    /// Loads a gold set from a CSV file
    ///
    /// The CSV file should have the following columns:
    /// - input_id: ID of the input identity
    /// - candidate_id: ID of the candidate identity
    /// - label: 1 for match, 0 for non-match
    ///
    /// Returns a Vec of GoldSetRecord
    pub fn load_gold_set_from_csv(file_path: &str) -> io::Result<Vec<GoldSetRecord>> {
        let file = File::open(file_path)?;
        let mut reader = ReaderBuilder::new()
            .has_headers(true)
            .from_reader(file);

        let mut records = Vec::new();

        for result in reader.records() {
            let record = result?;
            if record.len() < 3 {
                continue; // Skip records with insufficient fields
            }

            let input_id = record[0].to_string();
            let candidate_id = record[1].to_string();
            let label = record[2].parse::<u8>().unwrap_or(0);

            records.push(GoldSetRecord {
                input_id,
                candidate_id,
                is_match: label == 1,
            });
        }

        Ok(records)
    }

    /// Loads a gold set from a JSON file
    ///
    /// The JSON file should be an array of objects with the following fields:
    /// - input_id: ID of the input identity
    /// - candidate_id: ID of the candidate identity
    /// - label: 1 for match, 0 for non-match
    ///
    /// Returns a Vec of GoldSetRecord
    pub fn load_gold_set_from_json(file_path: &str) -> io::Result<Vec<GoldSetRecord>> {
        let mut file = File::open(file_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let json: Value = serde_json::from_str(&contents)?;
        let mut records = Vec::new();

        if let Value::Array(array) = json {
            for item in array {
                if let (Some(input_id), Some(candidate_id), Some(label)) = (
                    item.get("input_id").and_then(Value::as_str),
                    item.get("candidate_id").and_then(Value::as_str),
                    item.get("label").and_then(Value::as_u64),
                ) {
                    records.push(GoldSetRecord {
                        input_id: input_id.to_string(),
                        candidate_id: candidate_id.to_string(),
                        is_match: label == 1,
                    });
                }
            }
        }

        Ok(records)
    }

    /// Finds an identity by ID in a dictionary
    fn find_identity_by_id(dictionary: &Option<Box<IdentityNode>>, id: &str) -> Option<GoldSetIdentity> {
        let mut current = dictionary.as_ref().map(|b| b.as_ref());

        while let Some(node) = current {
            // In a real system, you would have an ID field in IdentityNode
            // For this example, we'll use a combination of fields as an ID
            let node_id = format!(
                "{}{}{}{}{}{}",
                node.first_name,
                node.last_name,
                node.father_name,
                node.grandfather_name,
                node.mother_last_name,
                node.mother_name
            );

            if node_id == id {
                return Some(GoldSetIdentity {
                    first_name: node.first_name.clone(),
                    last_name: node.last_name.clone(),
                    father_name: node.father_name.clone(),
                    grandfather_name: node.grandfather_name.clone(),
                    mother_last_name: node.mother_last_name.clone(),
                    mother_name: node.mother_name.clone(),
                    dob: node.dob,
                    sex: node.sex,
                    place_of_birth: node.place_of_birth.clone(),
                });
            }

            current = node.next_identity.as_ref().map(|b| b.as_ref());
        }

        None
    }

    /// Loads a gold set and returns a Vec of (GoldSetIdentity, GoldSetIdentity, bool) tuples
    ///
    /// The file extension is used to determine the file format:
    /// - .csv: CSV format
    /// - .json: JSON format
    ///
    /// Returns a Vec of (GoldSetIdentity, GoldSetIdentity, bool) tuples
    pub fn load_gold_set(file_path: &str, dictionary: &Option<Box<IdentityNode>>) -> io::Result<Vec<(GoldSetIdentity, GoldSetIdentity, bool)>> {
        let path = Path::new(file_path);
        let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");

        let records = match extension.to_lowercase().as_str() {
            "csv" => load_gold_set_from_csv(file_path)?,
            "json" => load_gold_set_from_json(file_path)?,
            _ => return Err(io::Error::new(io::ErrorKind::InvalidInput, "Unsupported file format")),
        };

        let mut result = Vec::new();

        for record in records {
            if let (Some(input), Some(candidate)) = (
                find_identity_by_id(dictionary, &record.input_id),
                find_identity_by_id(dictionary, &record.candidate_id),
            ) {
                result.push((input, candidate, record.is_match));
            }
        }

        Ok(result)
    }

    /// Creates a sample CSV gold set file for testing
    pub fn create_sample_csv_gold_set(file_path: &str) -> io::Result<()> {
        use std::io::Write;

        let mut file = File::create(file_path)?;

        // Write header
        writeln!(file, "input_id,candidate_id,label")?;

        // Write some sample records
        writeln!(file, "id1,id2,1")?;
        writeln!(file, "id1,id3,0")?;
        writeln!(file, "id4,id5,1")?;

        Ok(())
    }

    /// Creates a sample JSON gold set file for testing
    pub fn create_sample_json_gold_set(file_path: &str) -> io::Result<()> {
        use std::io::Write;

        let mut file = File::create(file_path)?;

        // Write JSON array of objects
        let json = r#"[
            {"input_id": "id1", "candidate_id": "id2", "label": 1},
            {"input_id": "id1", "candidate_id": "id3", "label": 0},
            {"input_id": "id4", "candidate_id": "id5", "label": 1}
        ]"#;

        file.write_all(json.as_bytes())?;

        Ok(())
    }

    /// Test function to demonstrate the usage of the gold set loader
    pub fn test_gold_set_loader() -> io::Result<()> {
        use crate::utils::linked_list::rebuild_identity_dictionary;

        // Create sample gold set files
        let csv_path = "sample_gold_set.csv";
        let json_path = "sample_gold_set.json";

        create_sample_csv_gold_set(csv_path)?;
        create_sample_json_gold_set(json_path)?;

        // Create a sample dictionary with some identities
        let records = vec![
            // Normalized fields, DOB, sex, place of birth, original fields
            (
                "ahmed".to_string(), "ben ali".to_string(), "mohamed".to_string(), "saleh".to_string(), "trabelsi".to_string(), "fatma".to_string(),
                Some((15, 6, 1985)), 1, "tunis".to_string(),
                "أحمد".to_string(), "بن علي".to_string(), "محمد".to_string(), "صالح".to_string(), "طرابلسي".to_string(), "فاطمة".to_string(),
            ),
            (
                "salma".to_string(), "hasni".to_string(), "abdullah".to_string(), "mohamed".to_string(), "ben salem".to_string(), "leila".to_string(),
                Some((3, 9, 1990)), 2, "sfax".to_string(),
                "سلمى".to_string(), "حسني".to_string(), "عبد الله".to_string(), "محمد".to_string(), "بن سالم".to_string(), "ليلى".to_string(),
            ),
        ];

        let dictionary = rebuild_identity_dictionary(records);

        // Load gold sets from CSV and JSON
        println!("Loading gold set from CSV...");
        let csv_records = load_gold_set(csv_path, &dictionary)?;
        println!("Loaded {} records from CSV", csv_records.len());

        println!("Loading gold set from JSON...");
        let json_records = load_gold_set(json_path, &dictionary)?;
        println!("Loaded {} records from JSON", json_records.len());

        // Clean up
        std::fs::remove_file(csv_path)?;
        std::fs::remove_file(json_path)?;

        Ok(())
    }
