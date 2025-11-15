use bytes::Bytes;
use relish::{ParseError, ParseErrorKind, ParseResult, Relish, parse, to_vec};

fn assert_roundtrips<T: Relish + PartialEq + std::fmt::Debug>(cases: &[(ParseResult<T>, &[u8])]) {
    for (expected, bytes) in cases {
        let parse_result = parse::<T>(Bytes::from(bytes.to_vec()));
        assert_eq!(expected, &parse_result);
        if let Ok(val) = expected {
            assert_eq!(to_vec(val).as_deref(), Ok(*bytes));
        }
    }
}

#[test]
fn test_simple_struct() {
    #[derive(Debug, PartialEq, Relish)]
    struct Simple {
        #[relish(field_id = 0)]
        value: u32,
    }

    assert_roundtrips(&[(
        Ok(Simple { value: 42 }),
        &[0x11, 0x0C, 0x00, 0x04, 0x2A, 0x00, 0x00, 0x00],
    )]);
}

#[test]
fn test_multiple_fields() {
    #[derive(Debug, PartialEq, Relish)]
    struct MultiField {
        #[relish(field_id = 0)]
        a: u32,
        #[relish(field_id = 1)]
        b: String,
        #[relish(field_id = 5)]
        c: bool,
    }

    assert_roundtrips(&[(
        Ok(MultiField {
            a: 42,
            b: "hello".to_string(),
            c: true,
        }),
        &[
            0x11, 0x22, 0x00, 0x04, 0x2A, 0x00, 0x00, 0x00, 0x01, 0x0E, 0x0A, b'h', b'e', b'l',
            b'l', b'o', 0x05, 0x01, 0xFF,
        ],
    )]);
}

#[test]
fn test_optional_fields() {
    #[derive(Debug, PartialEq, Relish)]
    struct WithOption {
        #[relish(field_id = 0)]
        required: u32,
        #[relish(field_id = 1)]
        optional: Option<u32>,
    }

    assert_roundtrips(&[
        (
            Ok(WithOption {
                required: 10,
                optional: Some(20),
            }),
            &[
                0x11, 0x18, 0x00, 0x04, 0x0A, 0x00, 0x00, 0x00, 0x01, 0x04, 0x14, 0x00, 0x00, 0x00,
            ],
        ),
        (
            Ok(WithOption {
                required: 10,
                optional: None,
            }),
            &[0x11, 0x0C, 0x00, 0x04, 0x0A, 0x00, 0x00, 0x00],
        ),
    ]);
}

#[test]
fn test_skip_field() {
    #[derive(Debug, PartialEq, Relish)]
    struct WithSkip {
        #[relish(field_id = 0)]
        included: u32,
        #[relish(skip)]
        skipped: String,
    }

    let value = WithSkip {
        included: 42,
        skipped: "not serialized".to_string(),
    };
    let serialized = to_vec(&value).unwrap();
    let parsed: WithSkip = parse(Bytes::from(serialized.clone())).unwrap();

    assert_eq!(parsed.included, 42);
    assert_eq!(parsed.skipped, "");
}

#[test]
fn test_empty_struct() {
    #[derive(Debug, PartialEq, Relish)]
    struct Empty {}

    assert_roundtrips(&[(Ok(Empty {}), &[0x11, 0x00])]);
}

#[test]
fn test_parse_with_unknown_fields() {
    #[derive(Debug, PartialEq, Relish)]
    struct Partial {
        #[relish(field_id = 0)]
        a: u32,
    }

    let data = &[
        0x11, 0x1C, 0x00, 0x04, 0x2A, 0x00, 0x00, 0x00, 0x02, 0x0E, 0x0A, b'h', b'e', b'l', b'l',
        b'o',
    ];

    let result: ParseResult<Partial> = parse(Bytes::from(data.to_vec()));
    assert_eq!(result, Ok(Partial { a: 42 }));
}

#[test]
fn test_parse_fields_not_in_order() {
    #[derive(Debug, PartialEq, Relish)]
    struct Ordered {
        #[relish(field_id = 0)]
        a: u32,
        #[relish(field_id = 1)]
        b: u32,
    }

    let data = &[
        0x11, 0x18, 0x01, 0x04, 0x14, 0x00, 0x00, 0x00, 0x00, 0x04, 0x0A, 0x00, 0x00, 0x00,
    ];

    let result: ParseResult<Ordered> = parse(Bytes::from(data.to_vec()));
    assert_eq!(
        result,
        Err(ParseError::new(ParseErrorKind::FieldOrderViolation {
            previous: 1,
            current: 0
        }))
    );
}

#[test]
fn test_nested_structs() {
    #[derive(Debug, PartialEq, Relish)]
    struct Inner {
        #[relish(field_id = 0)]
        value: u32,
    }

    #[derive(Debug, PartialEq, Relish)]
    struct Outer {
        #[relish(field_id = 0)]
        inner: Inner,
        #[relish(field_id = 1)]
        other: u32,
    }

    assert_roundtrips(&[(
        Ok(Outer {
            inner: Inner { value: 10 },
            other: 20,
        }),
        &[
            0x11, 0x1E, 0x00, 0x11, 0x0C, 0x00, 0x04, 0x0A, 0x00, 0x00, 0x00, 0x01, 0x04, 0x14,
            0x00, 0x00, 0x00,
        ],
    )]);
}

#[test]
fn test_simple_enum() {
    #[derive(Debug, PartialEq, Relish)]
    enum SimpleEnum {
        #[relish(field_id = 0)]
        A(u32),
        #[relish(field_id = 1)]
        B(String),
    }

    assert_roundtrips(&[
        (
            Ok(SimpleEnum::A(42)),
            &[0x12, 0x0C, 0x00, 0x04, 0x2A, 0x00, 0x00, 0x00],
        ),
        (
            Ok(SimpleEnum::B("hello".to_string())),
            &[0x12, 0x10, 0x01, 0x0E, 0x0A, b'h', b'e', b'l', b'l', b'o'],
        ),
    ]);
}

#[test]
fn test_enum_with_nested_struct() {
    #[derive(Debug, PartialEq, Relish)]
    struct Inner {
        #[relish(field_id = 0)]
        value: u32,
    }

    #[derive(Debug, PartialEq, Relish)]
    enum EnumWithStruct {
        #[relish(field_id = 0)]
        Simple(u32),
        #[relish(field_id = 1)]
        Complex(Inner),
    }

    assert_roundtrips(&[
        (
            Ok(EnumWithStruct::Simple(10)),
            &[0x12, 0x0C, 0x00, 0x04, 0x0A, 0x00, 0x00, 0x00],
        ),
        (
            Ok(EnumWithStruct::Complex(Inner { value: 20 })),
            &[
                0x12, 0x12, 0x01, 0x11, 0x0C, 0x00, 0x04, 0x14, 0x00, 0x00, 0x00,
            ],
        ),
    ]);
}

#[test]
fn test_enum_unknown_variant() {
    #[derive(Debug, PartialEq, Relish)]
    enum SimpleEnum {
        #[relish(field_id = 0)]
        A(u32),
    }

    let data = &[0x12, 0x0C, 0x05, 0x04, 0x2A, 0x00, 0x00, 0x00];

    let result: ParseResult<SimpleEnum> = parse(Bytes::from(data.to_vec()));
    assert_eq!(
        result,
        Err(ParseError::new(ParseErrorKind::UnknownVariant(5)))
    );
}

#[test]
fn test_nested_enums() {
    #[derive(Debug, PartialEq, Relish)]
    enum Inner {
        #[relish(field_id = 0)]
        X(u32),
        #[relish(field_id = 1)]
        Y(String),
    }

    #[derive(Debug, PartialEq, Relish)]
    enum Outer {
        #[relish(field_id = 0)]
        Nested(Inner),
        #[relish(field_id = 1)]
        Value(u32),
    }

    assert_roundtrips(&[
        (
            Ok(Outer::Nested(Inner::X(42))),
            &[
                0x12, 0x12, 0x00, 0x12, 0x0C, 0x00, 0x04, 0x2A, 0x00, 0x00, 0x00,
            ],
        ),
        (
            Ok(Outer::Value(10)),
            &[0x12, 0x0C, 0x01, 0x04, 0x0A, 0x00, 0x00, 0x00],
        ),
    ]);
}
