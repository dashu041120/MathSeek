use crate::*;

/// Example usage of the MathSeek data models
pub fn example_usage() {
    // Create a default configuration
    let mut config = AppConfig::default();
    config.api_endpoint = "https://api.openai.com/v1".to_string();
    config.api_key = "sk-example-key".to_string();
    
    println!("Created config: {:?}", config);
    
    // Validate the configuration
    match config.validate() {
        Ok(()) => println!("Configuration is valid"),
        Err(e) => println!("Configuration error: {}", e),
    }
    
    // Create a single formula result
    let formula_result = FormulaResult::new_single_formula(
        "E = mc^2".to_string(),
        0.95
    );
    
    println!("Formula result: {:?}", formula_result);
    
    // Create a document with multiple sections
    let mut document = DocumentContent::new(Some("Physics Equations".to_string()));
    
    let mut energy_section = DocumentSection::new(
        Some("Energy Equations".to_string()),
        "This section covers fundamental energy equations in physics.".to_string()
    );
    
    energy_section.add_formula(FormulaBlock::new(
        "E = mc^2".to_string(),
        45, // position in text
        false // not inline
    ));
    
    energy_section.add_formula(FormulaBlock::new(
        "KE = \\frac{1}{2}mv^2".to_string(),
        120,
        false
    ));
    
    document.add_section(energy_section);
    
    let mut motion_section = DocumentSection::new(
        Some("Motion Equations".to_string()),
        "Basic kinematic equations for motion with constant acceleration.".to_string()
    );
    
    motion_section.add_formula(FormulaBlock::new(
        "v = u + at".to_string(),
        65,
        true // inline formula
    ));
    
    document.add_section(motion_section);
    
    // Create a document formula result
    let document_result = FormulaResult::new_document(
        "Combined physics equations".to_string(),
        0.88,
        document
    );
    
    println!("Document result: {:?}", document_result);
    
    // Demonstrate serialization
    match serde_json::to_string_pretty(&document_result) {
        Ok(json) => println!("Serialized document result:\n{}", json),
        Err(e) => println!("Serialization error: {}", e),
    }
    
    // Demonstrate analysis result
    let analysis = AnalysisResult {
        formula_type: "Energy Equation".to_string(),
        description: "Einstein's mass-energy equivalence formula".to_string(),
        usage: "Used in nuclear physics and relativity calculations".to_string(),
        examples: vec![
            "Nuclear reactions".to_string(),
            "Particle physics".to_string(),
            "Cosmology".to_string(),
        ],
    };
    
    println!("Analysis result: {:?}", analysis);
    
    // Demonstrate format conversions
    let input_type = InputType::SingleFormula;
    let input_string: String = input_type.into();
    println!("InputType as string: {}", input_string);
    
    let parsed_input: Result<InputType, _> = input_string.try_into();
    match parsed_input {
        Ok(parsed) => println!("Parsed back to: {:?}", parsed),
        Err(e) => println!("Parse error: {}", e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_example_usage() {
        // This test just ensures the example code runs without panicking
        example_usage();
    }
}