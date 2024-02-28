#[macro_export]
macro_rules! insert_paths {
    ($openapi:expr, $($feature:ty),*) => {{
        $(
            let feat_swagger_meta = <$feature>::create_swagger();
            $openapi
                .paths
                .paths
                .insert(feat_swagger_meta.key, feat_swagger_meta.value);
        )*
    }};
}

