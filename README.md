Meta-REST
=========

## Overview ##
When a new idea comes to mind it's time to make the first prototype. And if that idea is about web it will definitely involve
server side coding. One of the options is REST service. So service consists of the bunch of resources. And these resources are
quite similar to each other. For each resource you have to implement following:
- storing in storage
- POST object to resource to create new
- GET a list of resources
- GET a list of resources filtered by some criterias
- GET a specific resource
- PUT request to update specific resource
- DELETE some resource
- define security policy for each resource
- validate incoming data

Most of these tasks have the same solutions and development is starting to remind "Groundhog Day" movie. This project is initiated
to simplify that situation.

Each resource is defined by its properties. Instead of direct implementation of resource behavior in source code it could be 
presented with the set of valuable properties. The sum of properties represents the entire REST service. This representation
could be called meta-description. Meta-description could be defined with the JSON-document in language described below.

## Implementation

This library is implemented in **Rust** and provides a complete solution for defining REST resources through JSON meta-descriptions.

### Features

- **Resource Definitions**: Define resources using JSON meta-descriptions with fields, types, and validation rules
- **CRUD Operations**: Automatic POST (create), GET (read), PUT (update), and DELETE operations
- **Validation**: Built-in validation for required fields, data types, min/max constraints
- **Filtering**: Query resources with filters (equals, not equals, greater than, less than, contains)
- **Storage Abstraction**: Pluggable storage backend (includes in-memory implementation)
- **Security Policies**: Define authentication requirements and role-based access control
- **Type Safety**: Full Rust type safety with Serde for JSON serialization/deserialization

### Quick Start

Add to your `Cargo.toml`:
```toml
[dependencies]
meta_rest = "0.1.0"
```

### Example Usage

```rust
use meta_rest::{Field, InMemoryStorage, Resource, ResourceDefinition, ResourceManager, ValidationRule};
use std::collections::HashMap;

// Define a resource using meta-description
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
        // ... more fields
    ],
    security: None,
};

// Create a resource manager
let storage = InMemoryStorage::new();
let mut manager = ResourceManager::new(user_definition, storage);

// POST - Create a resource
let mut data = HashMap::new();
data.insert("name".to_string(), serde_json::json!("Alice"));
let resource = Resource { id: "1".to_string(), data };
manager.create(resource).unwrap();

// GET - Retrieve resources
let user = manager.get("1").unwrap();
let all_users = manager.list().unwrap();
```

See `examples/basic_usage.rs` for a complete working example.

### Running Examples

```bash
cargo run --example basic_usage
```

### Building and Testing

```bash
# Build the library
cargo build

# Run tests
cargo test

# Run with code coverage
cargo tarpaulin

# Format code
cargo fmt

# Run linter
cargo clippy
```

### CI/CD

This project includes a GitHub Actions workflow that automatically:
- Checks code formatting
- Runs Clippy linter
- Builds the project
- Runs all tests
- Generates documentation
- Measures code coverage

## License

This project is open source.
 
