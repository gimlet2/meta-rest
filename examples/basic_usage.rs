use meta_rest::{
    Field, Filter, InMemoryStorage, Resource, ResourceDefinition, ResourceManager, SecurityPolicy,
    ValidationRule,
};
use std::collections::HashMap;

fn main() {
    println!("=== Meta-REST Example ===\n");

    // Define a "users" resource using meta-description
    let user_definition = ResourceDefinition {
        name: "users".to_string(),
        fields: vec![
            Field {
                name: "name".to_string(),
                field_type: "string".to_string(),
                required: true,
                validation: Some(ValidationRule {
                    min: Some(3.0),
                    max: Some(50.0),
                    pattern: None,
                }),
            },
            Field {
                name: "age".to_string(),
                field_type: "number".to_string(),
                required: false,
                validation: Some(ValidationRule {
                    min: Some(0.0),
                    max: Some(150.0),
                    pattern: None,
                }),
            },
            Field {
                name: "email".to_string(),
                field_type: "string".to_string(),
                required: true,
                validation: None,
            },
        ],
        security: Some(SecurityPolicy {
            require_auth: true,
            allowed_roles: Some(vec!["admin".to_string(), "user".to_string()]),
        }),
    };

    // Serialize definition to JSON
    let json = serde_json::to_string_pretty(&user_definition).unwrap();
    println!("Resource Definition (JSON):");
    println!("{}\n", json);

    // Create a resource manager with in-memory storage
    let storage = InMemoryStorage::new();
    let mut manager = ResourceManager::new(user_definition, storage);

    // POST - Create resources
    println!("Creating resources...");
    let user1 = create_user("1", "Alice Johnson", 28.0, "alice@example.com");
    let user2 = create_user("2", "Bob Smith", 35.0, "bob@example.com");
    let user3 = create_user("3", "Charlie Brown", 42.0, "charlie@example.com");

    manager.create(user1).unwrap();
    manager.create(user2).unwrap();
    manager.create(user3).unwrap();
    println!("Created 3 users\n");

    // GET - Retrieve a specific resource
    println!("Getting user with id '1':");
    let user = manager.get("1").unwrap();
    println!("{}\n", serde_json::to_string_pretty(&user).unwrap());

    // GET - List all resources
    println!("Listing all users:");
    let all_users = manager.list().unwrap();
    println!("Found {} users\n", all_users.len());

    // GET - Filter resources
    println!("Filtering users with age > 30:");
    let filters = vec![Filter {
        field: "age".to_string(),
        operator: "gt".to_string(),
        value: serde_json::Value::Number(serde_json::Number::from_f64(30.0).unwrap()),
    }];
    let filtered = manager.list_filtered(&filters).unwrap();
    for user in &filtered {
        println!(
            "  - {} (age: {})",
            user.data.get("name").unwrap().as_str().unwrap(),
            user.data.get("age").unwrap().as_f64().unwrap()
        );
    }
    println!();

    // PUT - Update a resource
    println!("Updating user '1'...");
    let updated_user = create_user("1", "Alice Johnson-Smith", 29.0, "alice.smith@example.com");
    manager.update("1", updated_user).unwrap();
    let user = manager.get("1").unwrap();
    println!(
        "Updated: {}\n",
        serde_json::to_string_pretty(&user).unwrap()
    );

    // DELETE - Remove a resource
    println!("Deleting user '3'...");
    manager.delete("3").unwrap();
    let all_users = manager.list().unwrap();
    println!("Remaining users: {}\n", all_users.len());

    // Validation example - this will fail
    println!("Testing validation (this should fail):");
    let invalid_user = create_user("4", "Jo", 200.0, "jo@example.com"); // Name too short, age too high
    match manager.create(invalid_user) {
        Ok(_) => println!("Unexpected success"),
        Err(e) => println!("Validation error (expected): {}\n", e),
    }

    println!("=== Example completed successfully! ===");
}

fn create_user(id: &str, name: &str, age: f64, email: &str) -> Resource {
    let mut data = HashMap::new();
    data.insert(
        "name".to_string(),
        serde_json::Value::String(name.to_string()),
    );
    data.insert(
        "age".to_string(),
        serde_json::Value::Number(serde_json::Number::from_f64(age).unwrap()),
    );
    data.insert(
        "email".to_string(),
        serde_json::Value::String(email.to_string()),
    );

    Resource {
        id: id.to_string(),
        data,
    }
}
