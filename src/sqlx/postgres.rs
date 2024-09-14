use sqlx::postgres::{PgHasArrayType, PgTypeInfo};
use sqlx::{Decode, Encode, Postgres, Type};

use crate::FractionalIndex;

impl<'r> Decode<'r, Postgres> for FractionalIndex {
    fn decode(value: sqlx::postgres::PgValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let bytes = <&[u8] as Decode<Postgres>>::decode(value)?;
        Ok(FractionalIndex::from_bytes(bytes.to_vec())?)
    }
}

impl Type<Postgres> for FractionalIndex {
    fn type_info() -> PgTypeInfo {
        <&[u8] as Type<Postgres>>::type_info()
    }
}

impl Encode<'_, Postgres> for FractionalIndex {
    fn encode_by_ref(
        &self,
        buf: &mut sqlx::postgres::PgArgumentBuffer,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        <&[u8] as Encode<Postgres>>::encode(&self[..], buf)
    }
}

impl PgHasArrayType for FractionalIndex {
    fn array_type_info() -> PgTypeInfo {
        <&[u8] as PgHasArrayType>::array_type_info()
    }
}
