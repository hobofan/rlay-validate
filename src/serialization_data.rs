use rlay_ontology::prelude::*;
use multicodec::{MultiCodec, Codec};
use snafu::Snafu;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Could not parse data value as Multicodec: {}", cause))]
    MulticodecParseError{ cause: multicodec::Error },
    #[snafu(display("Unsupported codec for data value: {:?}", codec))]
    UnsupportedCodec { codec: multicodec::Codec },
    #[snafu(display("Undecodable CBOR: {:?}", cause))]
    UndecodableCbor { cause: cbor::CborError },
}

/// Checks that the data fields of an entity are in the proper format
/// of a Multicodec encoded serialization format (CBOR, JSON, etc.), and that
/// the encoded data is deserializable.
///
/// Currently only supports CBOR.
#[derive(Debug, Default)]
pub struct SerializationFormatDataFields;

impl SerializationFormatDataFields {
    pub fn validate(&self, entity: &Entity) -> Result<(), Error> {
        match entity {
            Entity::Annotation(entity) => {
                Self::validate_field(&entity.value)?;
            }
            Entity::DataPropertyAssertion(entity) => {
                if let Some(ref value) = entity.target {
                    Self::validate_field(value)?;
                }
            }
            Entity::NegativeDataPropertyAssertion(entity) => {
                if let Some(ref value) = entity.target {
                    Self::validate_field(value)?;
                }
            }
            Entity::AnnotationAssertion(entity) => {
                if let Some(ref value) = entity.value {
                    Self::validate_field(value)?;
                }
            }
            Entity::NegativeAnnotationAssertion(entity) => {
                if let Some(ref value) = entity.value {
                    Self::validate_field(value)?;
                }
            }
            _ => (),
        }

        Ok(())
    }

    fn validate_field(data: &[u8]) -> Result<(), Error> {
        let parsed = MultiCodec::from(data).map_err(|e| Error::MulticodecParseError{ cause: e })?;
        match parsed.codec {
            Codec::Cbor => Self::validate_cbor_value(parsed.data),
            other => Err(Error::UnsupportedCodec { codec: other }),
        }?;

        Ok(())
    }

    fn validate_cbor_value(data: &[u8]) -> Result<(), Error> {
        let _: Vec<cbor::Cbor> = cbor::Decoder::from_bytes(data).items().collect::<Result<_, _>>().map_err(|e| Error::UndecodableCbor { cause: e })?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex_literal::hex;

    #[test]
    fn simple_annotation_cbor() {
        let entity: Entity = Annotation {
            annotations: vec![],
            property: vec![],
            // CBOR: true
            value: MultiCodec::new(Codec::Cbor, &hex!("f5").to_vec()).pack(),
        }.into();

        let validator = SerializationFormatDataFields::default();
        assert!(validator.validate(&entity).is_ok());
    }

    #[test]
    fn simple_annotation_wrong_codec() {
        let entity: Entity = Annotation {
            annotations: vec![],
            property: vec![],
            // Some Protobuf prefixed data
            value: MultiCodec::new(Codec::Protobuf, &hex!("f5").to_vec()).pack(),
        }.into();

        let validator = SerializationFormatDataFields::default();
        assert!(validator.validate(&entity).is_err());
    }

    #[test]
    fn simple_annotation_cbor_undecodable() {
        let entity: Entity = Annotation {
            annotations: vec![],
            property: vec![],
            // CBOR: undecodable data
            value: MultiCodec::new(Codec::Cbor, &hex!("f9").to_vec()).pack(),
        }.into();

        let validator = SerializationFormatDataFields::default();
        assert!(validator.validate(&entity).is_err());
    }
}
