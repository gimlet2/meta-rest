//! Meta-REST: A library for defining REST resources through meta-descriptions
//!
//! This library allows you to define REST API resources using JSON meta-descriptions
//! instead of implementing each resource manually. It provides automatic CRUD operations,
//! validation, filtering, and storage management.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

/// Represents a field in a resource definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Field {
    /// Name of the field
    pub name: String,
    /// Type of the field (e.g., "string", "number", "boolean")
    pub field_type: String,
    /// Whether the field is required
    pub required: bool,
    /// Optional validation rules
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validation: Option<ValidationRule>,
}

/// Validation rules for fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    /// Minimum value/length
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<f64>,
    /// Maximum value/length
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<f64>,
    /// Regex pattern for string validation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,
}

/// Security policy for a resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityPolicy {
    /// Whether authentication is required
    pub require_auth: bool,
    /// Allowed roles for access
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_roles: Option<Vec<String>>,
}

/// Resource meta-description defining the structure and behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceDefinition {
    /// Name of the resource
    pub name: String,
    /// Fields that make up the resource
    pub fields: Vec<Field>,
    /// Security policy for the resource
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security: Option<SecurityPolicy>,
}

/// A resource instance with dynamic data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    /// Unique identifier
    pub id: String,
    /// Resource data as key-value pairs
    pub data: HashMap<String, serde_json::Value>,
}

/// Filter criteria for querying resources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Filter {
    /// Field name to filter on
    pub field: String,
    /// Filter operator (e.g., "eq", "gt", "lt", "contains")
    pub operator: String,
    /// Value to compare against
    pub value: serde_json::Value,
}

/// Error types for meta-REST operations
#[derive(Debug)]
pub enum MetaRestError {
    /// Resource not found
    NotFound(String),
    /// Validation failed
    ValidationError(String),
    /// Storage error
    StorageError(String),
    /// Invalid operation
    InvalidOperation(String),
}

impl fmt::Display for MetaRestError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MetaRestError::NotFound(msg) => write!(f, "Not found: {}", msg),
            MetaRestError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            MetaRestError::StorageError(msg) => write!(f, "Storage error: {}", msg),
            MetaRestError::InvalidOperation(msg) => write!(f, "Invalid operation: {}", msg),
        }
    }
}

impl Error for MetaRestError {}

/// Storage abstraction for resource persistence
pub trait Storage: Send + Sync {
    /// Create a new resource
    fn create(&mut self, resource: Resource) -> Result<Resource, MetaRestError>;

    /// Get a resource by ID
    fn get(&self, id: &str) -> Result<Resource, MetaRestError>;

    /// Get all resources
    fn list(&self) -> Result<Vec<Resource>, MetaRestError>;

    /// Update a resource
    fn update(&mut self, id: &str, resource: Resource) -> Result<Resource, MetaRestError>;

    /// Delete a resource
    fn delete(&mut self, id: &str) -> Result<(), MetaRestError>;

    /// Filter resources based on criteria
    fn filter(&self, filters: &[Filter]) -> Result<Vec<Resource>, MetaRestError>;
}

/// In-memory storage implementation
#[derive(Debug, Default)]
pub struct InMemoryStorage {
    resources: HashMap<String, Resource>,
}

impl InMemoryStorage {
    /// Create a new in-memory storage
    pub fn new() -> Self {
        Self {
            resources: HashMap::new(),
        }
    }

    fn matches_filter(resource: &Resource, filter: &Filter) -> bool {
        if let Some(value) = resource.data.get(&filter.field) {
            match filter.operator.as_str() {
                "eq" => value == &filter.value,
                "ne" => value != &filter.value,
                "gt" => {
                    if let (Some(v1), Some(v2)) = (value.as_f64(), filter.value.as_f64()) {
                        v1 > v2
                    } else {
                        false
                    }
                }
                "lt" => {
                    if let (Some(v1), Some(v2)) = (value.as_f64(), filter.value.as_f64()) {
                        v1 < v2
                    } else {
                        false
                    }
                }
                "contains" => {
                    if let (Some(v1), Some(v2)) = (value.as_str(), filter.value.as_str()) {
                        v1.contains(v2)
                    } else {
                        false
                    }
                }
                _ => false,
            }
        } else {
            false
        }
    }
}

impl Storage for InMemoryStorage {
    fn create(&mut self, resource: Resource) -> Result<Resource, MetaRestError> {
        if self.resources.contains_key(&resource.id) {
            return Err(MetaRestError::InvalidOperation(format!(
                "Resource with id '{}' already exists",
                resource.id
            )));
        }
        self.resources.insert(resource.id.clone(), resource.clone());
        Ok(resource)
    }

    fn get(&self, id: &str) -> Result<Resource, MetaRestError> {
        self.resources
            .get(id)
            .cloned()
            .ok_or_else(|| MetaRestError::NotFound(format!("Resource with id '{}' not found", id)))
    }

    fn list(&self) -> Result<Vec<Resource>, MetaRestError> {
        Ok(self.resources.values().cloned().collect())
    }

    fn update(&mut self, id: &str, resource: Resource) -> Result<Resource, MetaRestError> {
        if !self.resources.contains_key(id) {
            return Err(MetaRestError::NotFound(format!(
                "Resource with id '{}' not found",
                id
            )));
        }
        self.resources.insert(id.to_string(), resource.clone());
        Ok(resource)
    }

    fn delete(&mut self, id: &str) -> Result<(), MetaRestError> {
        self.resources.remove(id).ok_or_else(|| {
            MetaRestError::NotFound(format!("Resource with id '{}' not found", id))
        })?;
        Ok(())
    }

    fn filter(&self, filters: &[Filter]) -> Result<Vec<Resource>, MetaRestError> {
        let results: Vec<Resource> = self
            .resources
            .values()
            .filter(|resource| {
                filters
                    .iter()
                    .all(|filter| Self::matches_filter(resource, filter))
            })
            .cloned()
            .collect();
        Ok(results)
    }
}

/// Resource manager that handles CRUD operations with validation
pub struct ResourceManager<S: Storage> {
    definition: ResourceDefinition,
    storage: S,
}

impl<S: Storage> ResourceManager<S> {
    /// Create a new resource manager with a definition and storage backend
    pub fn new(definition: ResourceDefinition, storage: S) -> Self {
        Self {
            definition,
            storage,
        }
    }

    /// Validate a resource against the definition
    pub fn validate(&self, resource: &Resource) -> Result<(), MetaRestError> {
        // Check required fields
        for field in &self.definition.fields {
            if field.required && !resource.data.contains_key(&field.name) {
                return Err(MetaRestError::ValidationError(format!(
                    "Required field '{}' is missing",
                    field.name
                )));
            }

            // Validate field type and rules if present
            if let Some(value) = resource.data.get(&field.name) {
                // Type checking
                let valid_type = match field.field_type.as_str() {
                    "string" => value.is_string(),
                    "number" => value.is_number(),
                    "boolean" => value.is_boolean(),
                    "array" => value.is_array(),
                    "object" => value.is_object(),
                    _ => true, // Unknown types are allowed
                };

                if !valid_type {
                    return Err(MetaRestError::ValidationError(format!(
                        "Field '{}' has invalid type, expected '{}'",
                        field.name, field.field_type
                    )));
                }

                // Validation rules
                if let Some(rules) = &field.validation {
                    if let Some(min) = rules.min {
                        if field.field_type == "number" {
                            if let Some(num) = value.as_f64() {
                                if num < min {
                                    return Err(MetaRestError::ValidationError(format!(
                                        "Field '{}' value {} is less than minimum {}",
                                        field.name, num, min
                                    )));
                                }
                            }
                        } else if field.field_type == "string" {
                            if let Some(s) = value.as_str() {
                                if s.len() < min as usize {
                                    return Err(MetaRestError::ValidationError(format!(
                                        "Field '{}' length is less than minimum {}",
                                        field.name, min
                                    )));
                                }
                            }
                        }
                    }

                    if let Some(max) = rules.max {
                        if field.field_type == "number" {
                            if let Some(num) = value.as_f64() {
                                if num > max {
                                    return Err(MetaRestError::ValidationError(format!(
                                        "Field '{}' value {} is greater than maximum {}",
                                        field.name, num, max
                                    )));
                                }
                            }
                        } else if field.field_type == "string" {
                            if let Some(s) = value.as_str() {
                                if s.len() > max as usize {
                                    return Err(MetaRestError::ValidationError(format!(
                                        "Field '{}' length is greater than maximum {}",
                                        field.name, max
                                    )));
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// POST - Create a new resource
    pub fn create(&mut self, resource: Resource) -> Result<Resource, MetaRestError> {
        self.validate(&resource)?;
        self.storage.create(resource)
    }

    /// GET - Retrieve a specific resource
    pub fn get(&self, id: &str) -> Result<Resource, MetaRestError> {
        self.storage.get(id)
    }

    /// GET - List all resources
    pub fn list(&self) -> Result<Vec<Resource>, MetaRestError> {
        self.storage.list()
    }

    /// GET - List resources with filters
    pub fn list_filtered(&self, filters: &[Filter]) -> Result<Vec<Resource>, MetaRestError> {
        self.storage.filter(filters)
    }

    /// PUT - Update a resource
    pub fn update(&mut self, id: &str, resource: Resource) -> Result<Resource, MetaRestError> {
        self.validate(&resource)?;
        self.storage.update(id, resource)
    }

    /// DELETE - Delete a resource
    pub fn delete(&mut self, id: &str) -> Result<(), MetaRestError> {
        self.storage.delete(id)
    }

    /// Get the resource definition
    pub fn definition(&self) -> &ResourceDefinition {
        &self.definition
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_definition() -> ResourceDefinition {
        ResourceDefinition {
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
        }
    }

    fn create_test_resource(id: &str, name: &str, age: f64, email: &str) -> Resource {
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

    #[test]
    fn test_resource_definition_serialization() {
        let def = create_test_definition();
        let json = serde_json::to_string(&def).unwrap();
        let deserialized: ResourceDefinition = serde_json::from_str(&json).unwrap();
        assert_eq!(def.name, deserialized.name);
        assert_eq!(def.fields.len(), deserialized.fields.len());
    }

    #[test]
    fn test_create_resource() {
        let def = create_test_definition();
        let storage = InMemoryStorage::new();
        let mut manager = ResourceManager::new(def, storage);

        let resource = create_test_resource("1", "John Doe", 30.0, "john@example.com");
        let result = manager.create(resource.clone());
        assert!(result.is_ok());

        let created = result.unwrap();
        assert_eq!(created.id, "1");
    }

    #[test]
    fn test_get_resource() {
        let def = create_test_definition();
        let storage = InMemoryStorage::new();
        let mut manager = ResourceManager::new(def, storage);

        let resource = create_test_resource("1", "John Doe", 30.0, "john@example.com");
        manager.create(resource).unwrap();

        let result = manager.get("1");
        assert!(result.is_ok());

        let retrieved = result.unwrap();
        assert_eq!(retrieved.id, "1");
        assert_eq!(
            retrieved.data.get("name").unwrap().as_str().unwrap(),
            "John Doe"
        );
    }

    #[test]
    fn test_get_nonexistent_resource() {
        let def = create_test_definition();
        let storage = InMemoryStorage::new();
        let manager = ResourceManager::new(def, storage);

        let result = manager.get("999");
        assert!(result.is_err());
        match result {
            Err(MetaRestError::NotFound(_)) => (),
            _ => panic!("Expected NotFound error"),
        }
    }

    #[test]
    fn test_list_resources() {
        let def = create_test_definition();
        let storage = InMemoryStorage::new();
        let mut manager = ResourceManager::new(def, storage);

        manager
            .create(create_test_resource(
                "1",
                "John Doe",
                30.0,
                "john@example.com",
            ))
            .unwrap();
        manager
            .create(create_test_resource(
                "2",
                "Jane Smith",
                25.0,
                "jane@example.com",
            ))
            .unwrap();

        let result = manager.list();
        assert!(result.is_ok());

        let resources = result.unwrap();
        assert_eq!(resources.len(), 2);
    }

    #[test]
    fn test_update_resource() {
        let def = create_test_definition();
        let storage = InMemoryStorage::new();
        let mut manager = ResourceManager::new(def, storage);

        let resource = create_test_resource("1", "John Doe", 30.0, "john@example.com");
        manager.create(resource).unwrap();

        let updated = create_test_resource("1", "John Smith", 31.0, "john.smith@example.com");
        let result = manager.update("1", updated);
        assert!(result.is_ok());

        let retrieved = manager.get("1").unwrap();
        assert_eq!(
            retrieved.data.get("name").unwrap().as_str().unwrap(),
            "John Smith"
        );
        assert_eq!(retrieved.data.get("age").unwrap().as_f64().unwrap(), 31.0);
    }

    #[test]
    fn test_delete_resource() {
        let def = create_test_definition();
        let storage = InMemoryStorage::new();
        let mut manager = ResourceManager::new(def, storage);

        let resource = create_test_resource("1", "John Doe", 30.0, "john@example.com");
        manager.create(resource).unwrap();

        let result = manager.delete("1");
        assert!(result.is_ok());

        let get_result = manager.get("1");
        assert!(get_result.is_err());
    }

    #[test]
    fn test_validation_required_fields() {
        let def = create_test_definition();
        let storage = InMemoryStorage::new();
        let mut manager = ResourceManager::new(def, storage);

        let mut data = HashMap::new();
        data.insert(
            "name".to_string(),
            serde_json::Value::String("John".to_string()),
        );
        // Missing required email field

        let resource = Resource {
            id: "1".to_string(),
            data,
        };

        let result = manager.create(resource);
        assert!(result.is_err());
        match result {
            Err(MetaRestError::ValidationError(msg)) => {
                assert!(msg.contains("email"));
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    #[test]
    fn test_validation_field_type() {
        let def = create_test_definition();
        let storage = InMemoryStorage::new();
        let mut manager = ResourceManager::new(def, storage);

        let mut data = HashMap::new();
        data.insert(
            "name".to_string(),
            serde_json::Value::String("John Doe".to_string()),
        );
        data.insert(
            "age".to_string(),
            serde_json::Value::String("thirty".to_string()),
        ); // Should be number
        data.insert(
            "email".to_string(),
            serde_json::Value::String("john@example.com".to_string()),
        );

        let resource = Resource {
            id: "1".to_string(),
            data,
        };

        let result = manager.create(resource);
        assert!(result.is_err());
        match result {
            Err(MetaRestError::ValidationError(msg)) => {
                assert!(msg.contains("age"));
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    #[test]
    fn test_validation_min_max_number() {
        let def = create_test_definition();
        let storage = InMemoryStorage::new();
        let mut manager = ResourceManager::new(def, storage);

        // Test minimum
        let resource = create_test_resource("1", "John Doe", -5.0, "john@example.com");
        let result = manager.create(resource);
        assert!(result.is_err());

        // Test maximum
        let resource = create_test_resource("2", "Jane Doe", 200.0, "jane@example.com");
        let result = manager.create(resource);
        assert!(result.is_err());
    }

    #[test]
    fn test_validation_min_max_string() {
        let def = create_test_definition();
        let storage = InMemoryStorage::new();
        let mut manager = ResourceManager::new(def, storage);

        // Test minimum length
        let resource = create_test_resource("1", "Jo", 30.0, "jo@example.com");
        let result = manager.create(resource);
        assert!(result.is_err());

        // Test maximum length
        let long_name = "A".repeat(100);
        let resource = create_test_resource("2", &long_name, 30.0, "test@example.com");
        let result = manager.create(resource);
        assert!(result.is_err());
    }

    #[test]
    fn test_filter_resources() {
        let def = create_test_definition();
        let storage = InMemoryStorage::new();
        let mut manager = ResourceManager::new(def, storage);

        manager
            .create(create_test_resource(
                "1",
                "John Doe",
                30.0,
                "john@example.com",
            ))
            .unwrap();
        manager
            .create(create_test_resource(
                "2",
                "Jane Smith",
                25.0,
                "jane@example.com",
            ))
            .unwrap();
        manager
            .create(create_test_resource(
                "3",
                "Bob Jones",
                35.0,
                "bob@example.com",
            ))
            .unwrap();

        // Filter by age greater than 28
        let filters = vec![Filter {
            field: "age".to_string(),
            operator: "gt".to_string(),
            value: serde_json::Value::Number(serde_json::Number::from_f64(28.0).unwrap()),
        }];

        let result = manager.list_filtered(&filters);
        assert!(result.is_ok());

        let filtered = result.unwrap();
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_filter_equals() {
        let def = create_test_definition();
        let storage = InMemoryStorage::new();
        let mut manager = ResourceManager::new(def, storage);

        manager
            .create(create_test_resource(
                "1",
                "John Doe",
                30.0,
                "john@example.com",
            ))
            .unwrap();
        manager
            .create(create_test_resource(
                "2",
                "Jane Smith",
                25.0,
                "jane@example.com",
            ))
            .unwrap();

        let filters = vec![Filter {
            field: "name".to_string(),
            operator: "eq".to_string(),
            value: serde_json::Value::String("John Doe".to_string()),
        }];

        let result = manager.list_filtered(&filters);
        assert!(result.is_ok());

        let filtered = result.unwrap();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "1");
    }

    #[test]
    fn test_filter_contains() {
        let def = create_test_definition();
        let storage = InMemoryStorage::new();
        let mut manager = ResourceManager::new(def, storage);

        manager
            .create(create_test_resource(
                "1",
                "John Doe",
                30.0,
                "john@example.com",
            ))
            .unwrap();
        manager
            .create(create_test_resource(
                "2",
                "Jane Doe",
                25.0,
                "jane@example.com",
            ))
            .unwrap();
        manager
            .create(create_test_resource(
                "3",
                "Bob Smith",
                35.0,
                "bob@example.com",
            ))
            .unwrap();

        let filters = vec![Filter {
            field: "name".to_string(),
            operator: "contains".to_string(),
            value: serde_json::Value::String("Doe".to_string()),
        }];

        let result = manager.list_filtered(&filters);
        assert!(result.is_ok());

        let filtered = result.unwrap();
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_multiple_filters() {
        let def = create_test_definition();
        let storage = InMemoryStorage::new();
        let mut manager = ResourceManager::new(def, storage);

        manager
            .create(create_test_resource(
                "1",
                "John Doe",
                30.0,
                "john@example.com",
            ))
            .unwrap();
        manager
            .create(create_test_resource(
                "2",
                "Jane Doe",
                25.0,
                "jane@example.com",
            ))
            .unwrap();
        manager
            .create(create_test_resource(
                "3",
                "Bob Doe",
                35.0,
                "bob@example.com",
            ))
            .unwrap();

        let filters = vec![
            Filter {
                field: "name".to_string(),
                operator: "contains".to_string(),
                value: serde_json::Value::String("Doe".to_string()),
            },
            Filter {
                field: "age".to_string(),
                operator: "gt".to_string(),
                value: serde_json::Value::Number(serde_json::Number::from_f64(28.0).unwrap()),
            },
        ];

        let result = manager.list_filtered(&filters);
        assert!(result.is_ok());

        let filtered = result.unwrap();
        assert_eq!(filtered.len(), 2); // John and Bob, not Jane (age 25)
    }
}
