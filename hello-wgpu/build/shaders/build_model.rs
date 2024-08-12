use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

// Define the BitSlice struct
#[derive(Debug)]
pub struct BitSlice {
    pub bit_start: u32,
    pub bit_length: u32,
    pub subbit_start: Option<u32>,
    pub subbit_length: Option<u32>,
}

// Define the Property struct
#[derive(Debug)]
pub struct Property {
    name: String,
    pub bit_slices: Vec<BitSlice>
}

#[derive(Debug)]
pub struct Model {
    pub name: String,
    pub properties: HashMap<String, Property>,
}

impl Model {
    pub fn get_num_u32s_to_fit_model(&self) -> usize {
        let mut num_bits = 0;
        for property in self.properties.iter() {
            for bit_slice in property.1.bit_slices.iter() {
                num_bits += bit_slice.bit_length;
            }
        }

        (num_bits + 31) as usize / 32
    }
}

// Function to parse the file and return a vector of Model objects
pub fn parse_file(filepath: &str) -> io::Result<Vec<Model>> {
    let path = Path::new(filepath);
    let file = File::open(&path)?;
    let reader = io::BufReader::new(file);

    let mut models: Vec<Model> = Vec::new();
    let mut current_model: Option<Model> = None;

    for line in reader.lines() {
        let line = line?;
        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts.is_empty() {
            continue; // Skip empty lines
        }

        if parts[0] == "MODEL" && parts[1] == "NAME" && parts.len() == 3 {
            // Create a new model
            if let Some(model) = current_model.take() {
                models.push(model);
            }
            current_model = Some(Model {
                name: parts[2].to_string(),
                properties: HashMap::new(),
            });
        } else if let Some(ref mut model) = current_model {
            // Process property lines
            if parts.len() < 3 {
                continue; // Skip malformed lines
            }

            let name = parts[0].to_string();
            let bit_start = parts[1].parse::<u32>().unwrap();
            let bit_length = parts[2].parse::<u32>().unwrap();

            let mut bit_slices = Vec::new();
            let mut i = 3;
            while i < parts.len() {
                let subbit_start = parts[i].parse::<u32>().unwrap();
                let subbit_length = parts[i + 1].parse::<u32>().unwrap();
                bit_slices.push(BitSlice {
                    bit_start,
                    bit_length,
                    subbit_start: Some(subbit_start),
                    subbit_length: Some(subbit_length),
                });
                i += 2;
            }

            if bit_slices.is_empty() {
                // If there are no subbits, create a single BitSlice without subbits
                bit_slices.push(BitSlice {
                    bit_start,
                    bit_length,
                    subbit_start: None,
                    subbit_length: None,
                });
            }

            // Update or insert the property
            let property = model.properties.entry(name.clone()).or_insert(Property {
                name,
                bit_slices: Vec::new()
            });

            property.bit_slices.extend(bit_slices);
        }
    }

    // Push the last model if exists
    if let Some(model) = current_model {
        models.push(model);
    }

    Ok(models)
}