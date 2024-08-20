pub trait Identifier {
    fn new() -> Self;
    fn uuid(&self) -> &uuid::Uuid;
}

/// Generate an X struct holding an uuid and generating it's default implementation
/// ```ignore
/// crate::model_id!(UserId, "usr");
/// ```
///
/// Will expand to
/// ```
/// #[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// pub struct UserId(uuid::Uuid);
///
/// impl std::fmt::Display for UserId {
///     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
///         write!(f, "{}-{}", "usr", self.0)
///     }
/// }
///
/// impl sora_model::id::Identifier for UserId {
///     fn new() -> Self {
///         Self(uuid::Uuid::now_v7())
///     }
///
///     fn uuid(&self) -> &uuid::Uuid {
///         &self.0
///     }
/// }
/// ```
#[macro_export]
macro_rules! model_id {
    ($model_name:ident, $repr:literal) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub struct $model_name(uuid::Uuid);

        impl std::fmt::Display for $model_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}-{}", $repr, self.0)
            }
        }

        impl crate::id::Identifier for $model_name {
            fn new() -> Self {
                Self(uuid::Uuid::now_v7())
            }

            fn uuid(&self) -> &uuid::Uuid {
                &self.0
            }
        }
    };
}

#[cfg(test)]
mod test {
    use super::Identifier;

    model_id!(TestId, "tst");

    #[test]
    pub fn test_identifiers_generation() {
        // Ensure this compiles and doesn't panic
        let id = TestId::new();

        id.uuid();
        id.to_string();
        let _ = id.clone();
        id.0;
    }
}
