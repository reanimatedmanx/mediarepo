use crate::file::File;
use crate::file_type::FileType;
use crate::namespace::Namespace;
use crate::storage::Storage;
use crate::tag::Tag;
use crate::thumbnail::Thumbnail;
use chrono::{Local, NaiveDateTime};
use mediarepo_core::error::{RepoError, RepoResult};
use mediarepo_core::itertools::Itertools;
use mediarepo_core::thumbnailer::ThumbnailSize;
use mediarepo_core::utils::parse_namespace_and_tag;
use mediarepo_database::get_database;
use sea_orm::DatabaseConnection;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::io::Cursor;
use std::iter::FromIterator;
use std::path::PathBuf;
use std::str::FromStr;
use tokio::fs::OpenOptions;
use tokio::io::BufReader;

#[derive(Clone)]
pub struct Repo {
    db: DatabaseConnection,
    main_storage: Option<Storage>,
    thumbnail_storage: Option<Storage>,
}

impl Repo {
    pub(crate) fn new(db: DatabaseConnection) -> Self {
        Self {
            db,
            main_storage: None,
            thumbnail_storage: None,
        }
    }

    /// Connects to the database with the given uri
    #[tracing::instrument(level = "debug")]
    pub async fn connect<S: AsRef<str> + Debug>(uri: S) -> RepoResult<Self> {
        let db = get_database(uri).await?;
        Ok(Self::new(db))
    }

    /// Returns the database of the repo for raw sql queries
    pub fn db(&self) -> &DatabaseConnection {
        &self.db
    }

    /// Returns all available storages
    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn storages(&self) -> RepoResult<Vec<Storage>> {
        Storage::all(self.db.clone()).await
    }

    /// Returns a storage by path
    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn storage_by_path<S: ToString + Debug>(
        &self,
        path: S,
    ) -> RepoResult<Option<Storage>> {
        Storage::by_path(self.db.clone(), path).await
    }

    /// Sets the main storage
    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn set_main_storage<S: ToString + Debug>(&mut self, path: S) -> RepoResult<()> {
        self.main_storage = Storage::by_name(self.db.clone(), path.to_string()).await?;
        Ok(())
    }

    /// Sets the default thumbnail storage
    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn set_thumbnail_storage<S: ToString + Debug>(&mut self, path: S) -> RepoResult<()> {
        self.thumbnail_storage = Storage::by_name(self.db.clone(), path.to_string()).await?;
        Ok(())
    }

    /// Adds a storage to the repository
    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn add_storage<S1: ToString + Debug, S2: ToString + Debug>(
        &self,
        name: S1,
        path: S2,
    ) -> RepoResult<Storage> {
        Storage::create(self.db.clone(), name, path).await
    }

    /// Returns a file by its mapped hash
    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn file_by_hash<S: AsRef<str> + Debug>(&self, hash: S) -> RepoResult<Option<File>> {
        File::by_hash(self.db.clone(), hash).await
    }

    /// Returns a file by id
    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn file_by_id(&self, id: i64) -> RepoResult<Option<File>> {
        File::by_id(self.db.clone(), id).await
    }

    /// Returns a list of all stored files
    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn files(&self) -> RepoResult<Vec<File>> {
        File::all(self.db.clone()).await
    }

    /// Finds all files by a list of tags
    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn find_files_by_tags(&self, tags: Vec<(String, bool)>) -> RepoResult<Vec<File>> {
        let parsed_tags = tags
            .iter()
            .map(|t| parse_namespace_and_tag(t.0.clone()))
            .collect();

        let db_tags = self.tags_by_names(parsed_tags).await?;
        let tag_map: HashMap<String, bool> = HashMap::from_iter(tags.into_iter());

        let tag_ids: Vec<(i64, bool)> = db_tags
            .into_iter()
            .map(|tag| {
                (
                    tag.id(),
                    tag_map
                        .get(&tag.normalized_name())
                        .cloned()
                        .unwrap_or(false),
                )
            })
            .collect();

        File::find_by_tags(self.db.clone(), tag_ids).await
    }

    /// Adds a file from bytes to the database
    #[tracing::instrument(level = "debug", skip(self, content))]
    pub async fn add_file(
        &self,
        mime_type: Option<String>,
        content: Vec<u8>,
        creation_time: NaiveDateTime,
        change_time: NaiveDateTime,
    ) -> RepoResult<File> {
        let storage = self.get_main_storage()?;
        let reader = Cursor::new(content);
        let hash = storage.store_entry(reader).await?;

        let (mime_type, file_type) = mime_type
            .and_then(|m| mime::Mime::from_str(&m).ok())
            .map(|m| (Some(m.to_string()), FileType::from(m)))
            .unwrap_or((None, FileType::Unknown));

        File::add(
            self.db.clone(),
            storage.id(),
            hash.id(),
            file_type,
            mime_type,
            creation_time,
            change_time,
        )
        .await
    }

    /// Adds a file to the database by its readable path in the file system
    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn add_file_by_path(&self, path: PathBuf) -> RepoResult<File> {
        let mime_match = mime_guess::from_path(&path).first();

        let (mime_type, file_type) = if let Some(mime) = mime_match {
            (Some(mime.clone().to_string()), FileType::from(mime))
        } else {
            (None, FileType::Unknown)
        };
        let os_file = OpenOptions::new().read(true).open(&path).await?;
        let reader = BufReader::new(os_file);

        let storage = self.get_main_storage()?;
        let hash = storage.store_entry(reader).await?;
        File::add(
            self.db.clone(),
            storage.id(),
            hash.id(),
            file_type,
            mime_type,
            Local::now().naive_local(),
            Local::now().naive_local(),
        )
        .await
    }

    /// Returns a thumbnail by its hash
    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn thumbnail_by_hash<S: AsRef<str> + Debug>(
        &self,
        hash: S,
    ) -> RepoResult<Option<Thumbnail>> {
        Thumbnail::by_hash(self.db.clone(), hash).await
    }

    /// Creates thumbnails of all sizes for a file
    #[tracing::instrument(level = "debug", skip(self, file))]
    pub async fn create_thumbnails_for_file(&self, file: &File) -> RepoResult<()> {
        let thumb_storage = self.get_thumbnail_storage()?;
        let size = ThumbnailSize::Medium;
        let (height, width) = size.dimensions();
        let thumbs = file.create_thumbnail([size]).await?;

        for thumb in thumbs {
            self.store_single_thumbnail(file, thumb_storage, height, width, thumb)
                .await?;
        }

        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self, file))]
    pub async fn create_file_thumbnail(
        &self,
        file: &File,
        size: ThumbnailSize,
    ) -> RepoResult<Thumbnail> {
        let thumb_storage = self.get_thumbnail_storage()?;
        let (height, width) = size.dimensions();
        let thumb = file
            .create_thumbnail([size])
            .await?
            .pop()
            .ok_or_else(|| RepoError::from("Failed to create thumbnail"))?;
        let thumbnail = self
            .store_single_thumbnail(file, thumb_storage, height, width, thumb)
            .await?;

        Ok(thumbnail)
    }

    async fn store_single_thumbnail(
        &self,
        file: &File,
        thumb_storage: &Storage,
        height: u32,
        width: u32,
        thumb: mediarepo_core::thumbnailer::Thumbnail,
    ) -> RepoResult<Thumbnail> {
        let mut buf = Vec::new();
        thumb.write_png(&mut buf)?;
        let hash = thumb_storage.store_entry(Cursor::new(buf)).await?;

        let thumbnail = Thumbnail::add(
            self.db.clone(),
            hash.id(),
            file.id(),
            thumb_storage.id(),
            height as i32,
            width as i32,
            Some(mime::IMAGE_PNG.to_string()),
        )
        .await?;

        Ok(thumbnail)
    }

    /// Returns all tags stored in the database
    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn tags(&self) -> RepoResult<Vec<Tag>> {
        Tag::all(self.db.clone()).await
    }

    /// Finds all tags by name
    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn tags_by_names(&self, tags: Vec<(Option<String>, String)>) -> RepoResult<Vec<Tag>> {
        Tag::all_by_name(self.db.clone(), tags).await
    }

    /// Finds all tags that are assigned to the given list of hashes
    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn find_tags_for_hashes(&self, hashes: Vec<String>) -> RepoResult<Vec<Tag>> {
        Tag::for_hash_list(self.db.clone(), hashes).await
    }

    /// Adds all tags that are not in the database to the database and returns the ones already existing as well
    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn add_all_tags(&self, tags: Vec<(Option<String>, String)>) -> RepoResult<Vec<Tag>> {
        let mut tags_to_add = tags.into_iter().unique().collect_vec();
        let mut namespaces_to_add = tags_to_add
            .iter()
            .filter_map(|(namespace, _)| namespace.clone())
            .unique()
            .collect_vec();

        let mut existing_namespaces =
            Namespace::all_by_name(self.db.clone(), namespaces_to_add.clone()).await?;
        {
            let existing_namespaces_set = existing_namespaces
                .iter()
                .map(|n| n.name().clone())
                .collect::<HashSet<String>>();
            namespaces_to_add.retain(|namespace| !existing_namespaces_set.contains(namespace));
        }
        existing_namespaces
            .append(&mut Namespace::add_all(self.db.clone(), namespaces_to_add).await?);

        let mut existing_tags = self.tags_by_names(tags_to_add.clone()).await?;
        {
            let existing_tags_set = existing_tags
                .iter()
                .map(|t| (t.namespace().map(|n| n.name().clone()), t.name().clone()))
                .collect::<HashSet<(Option<String>, String)>>();

            tags_to_add.retain(|t| !existing_tags_set.contains(t));
        }
        let namespace_map = existing_namespaces
            .into_iter()
            .map(|namespace| (namespace.name().clone(), namespace.id()))
            .collect::<HashMap<String, i64>>();
        let tags_to_add = tags_to_add
            .into_iter()
            .map(|(nsp, name)| (nsp.and_then(|n| namespace_map.get(&n)).map(|i| *i), name))
            .collect_vec();
        existing_tags.append(&mut Tag::add_all(self.db.clone(), tags_to_add).await?);

        Ok(existing_tags)
    }

    /// Adds or finds a tag
    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn add_or_find_tag<S: ToString + Debug>(&self, tag: S) -> RepoResult<Tag> {
        let (namespace, name) = parse_namespace_and_tag(tag.to_string());
        if let Some(namespace) = namespace {
            self.add_or_find_namespaced_tag(name, namespace).await
        } else {
            self.add_or_find_unnamespaced_tag(name).await
        }
    }

    /// Adds or finds an unnamespaced tag
    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn add_or_find_unnamespaced_tag(&self, name: String) -> RepoResult<Tag> {
        if let Some(tag) = Tag::by_name(self.db.clone(), &name, None).await? {
            Ok(tag)
        } else {
            self.add_unnamespaced_tag(name).await
        }
    }

    /// Adds an unnamespaced tag
    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn add_unnamespaced_tag(&self, name: String) -> RepoResult<Tag> {
        Tag::add(self.db.clone(), name, None).await
    }

    /// Adds or finds a namespaced tag
    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn add_or_find_namespaced_tag(
        &self,
        name: String,
        namespace: String,
    ) -> RepoResult<Tag> {
        if let Some(tag) = Tag::by_name(self.db.clone(), &name, Some(namespace.clone())).await? {
            Ok(tag)
        } else {
            self.add_namespaced_tag(name, namespace).await
        }
    }

    /// Adds a namespaced tag
    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn add_namespaced_tag(&self, name: String, namespace: String) -> RepoResult<Tag> {
        let namespace =
            if let Some(namespace) = Namespace::by_name(self.db.clone(), &namespace).await? {
                namespace
            } else {
                Namespace::add(self.db.clone(), namespace).await?
            };
        Tag::add(self.db.clone(), name, Some(namespace.id())).await
    }

    #[tracing::instrument(level = "trace", skip(self))]
    fn get_main_storage(&self) -> RepoResult<&Storage> {
        if let Some(storage) = &self.main_storage {
            Ok(storage)
        } else {
            Err(RepoError::from("No main storage configured."))
        }
    }

    #[tracing::instrument(level = "trace", skip(self))]
    fn get_thumbnail_storage(&self) -> RepoResult<&Storage> {
        if let Some(storage) = &self.thumbnail_storage {
            Ok(storage)
        } else {
            Err(RepoError::from("No thumbnail storage configured."))
        }
    }
}
