#[cfg(test)]
mod tests {
    use mdmodels_macro::parse_mdmodel;

    parse_mdmodel!("tests/data/model.md");

    #[test]
    fn test_model_builder() {
        // Build the 'Object' struct
        test::ObjectBuilder::default()
            .string_value("Hello, World!")
            .integer_value(42)
            .float_value(3.2)
            .boolean_value(true)
            .multiple_values(vec![1.12, 1.0])
            .nested_object(
                test::NestedBuilder::default()
                    .value("nested")
                    .build()
                    .unwrap(),
            )
            .multiple_nested_objects(vec![
                test::NestedBuilder::default()
                    .value("nested1")
                    .build()
                    .unwrap(),
                test::NestedBuilder::default()
                    .value("nested2")
                    .build()
                    .unwrap(),
            ])
            .enum_value(test::SomeEnum::Value)
            .build()
            .expect("Failed to build object");
    }

    #[test]
    fn test_model_non_builder() {
        // Build the 'Object' struct
        test::Object {
            additional_properties: None,
            string_value: Some("Hello, World!".to_string()),
            integer_value: Some(42),
            float_value: Some(3.2),
            boolean_value: Some(true),
            multiple_values: Some(vec![1.12, 1.0]),
            nested_object: Some(test::Nested {
                additional_properties: None,
                value: Some("nested".to_string()),
            }),
            multiple_nested_objects: vec![
                test::Nested {
                    additional_properties: None,
                    value: Some("nested1".to_string()),
                },
                test::Nested {
                    additional_properties: None,
                    value: Some("nested2".to_string()),
                },
            ],
            enum_value: Some(test::SomeEnum::Value),
        };
    }

    #[test]
    fn test_model_parser() {
        // Parse the 'Object' struct from a JSON string
        let json = r#"
            {
                "string_value": "Hello, World!",
                "integer_value": 42,
                "float_value": 3.2,
                "boolean_value": true,
                "multiple_values": [1.12, 1.0],
                "nested_object": {
                    "value": "nested"
                },
                "multiple_nested_objects": [
                    {
                        "value": "nested1"
                    },
                    {
                        "value": "nested2"
                    }
                ],
                "enum_value": "value"
            }
        "#;

        let object: test::Object = serde_json::from_str(json).expect("Failed to parse JSON");
        assert_eq!(object.string_value, Some("Hello, World!".to_string()));
        assert_eq!(object.integer_value, Some(42));
        assert_eq!(object.float_value, Some(3.2));
        assert_eq!(object.boolean_value, Some(true));
        assert_eq!(object.multiple_values, Some(vec![1.12, 1.0]));
        assert_eq!(
            object.nested_object.unwrap().value,
            Some("nested".to_string())
        );
        assert_eq!(object.multiple_nested_objects.len(), 2);
        assert_eq!(
            object.multiple_nested_objects[0].value,
            Some("nested1".to_string())
        );
        assert_eq!(
            object.multiple_nested_objects[1].value,
            Some("nested2".to_string())
        );
        assert_eq!(object.enum_value.unwrap(), test::SomeEnum::Value);
    }
}
