//! The collections a Qdrant holds, for `mash qdrant prune`: list them, count
//! their points and delete one. Nothing here decides what may be deleted; the
//! caller classifies every collection first and fails closed on any error.
use super::{AppError, Client};
use reqwest::Method;
use serde::Deserialize;

#[derive(Deserialize)]
struct Listing {
    collections: Vec<Named>,
}
#[derive(Deserialize)]
struct Named {
    name: String,
}
#[derive(Deserialize)]
struct Counts {
    points_count: Option<u64>,
}

impl Client {
    /// Every collection's name, whoever created it.
    pub async fn collection_names(&self) -> Result<Vec<String>, AppError> {
        let listing: Listing = self.call(Method::GET, "collections", None::<&()>).await?;
        Ok(listing.collections.into_iter().map(|c| c.name).collect())
    }

    /// How many points a collection holds, as Qdrant reports it.
    pub async fn collection_points(&self, name: &str) -> Result<Option<u64>, AppError> {
        let counts: Counts = self
            .call(Method::GET, &format!("collections/{name}"), None::<&()>)
            .await?;
        Ok(counts.points_count)
    }

    /// Delete one whole collection. Qdrant answers `true` when it was removed.
    pub async fn delete_collection(&self, name: &str) -> Result<(), AppError> {
        let removed: bool = self
            .call(Method::DELETE, &format!("collections/{name}"), None::<&()>)
            .await?;
        if removed {
            Ok(())
        } else {
            Err(AppError::configuration(format!(
                "Qdrant did not delete {name}; nothing after it was deleted"
            )))
        }
    }
}
