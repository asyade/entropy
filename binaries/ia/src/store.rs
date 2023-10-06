use crate::prelude::*;

pub const TREE_SETTINGS: &str = "____settings";
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct Store {
    db: sled::Db,
    opened_guys: Arc<RwLock<HashMap<String, GuyHandle>>>,
}

#[derive(Clone)]
pub struct GuyHandle {
    tree: sled::Tree,
    guy: Arc<RwLock<Guy>>,
}

impl Store {
    pub fn create(store_directory: &Path) -> Result<Self, anyhow::Error> {
        if store_directory.exists() {
            return Err(IaError::Message(format!("The file already exists: `{:?}`", store_directory)))?;
        }
        let store = sled::open(store_directory)?;
        let tree = store.open_tree(TREE_SETTINGS)?;
        tree.insert("version", VERSION)?;
        Ok(Self {
            db: store,
            opened_guys: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    pub fn open(store_directory: &Path) -> Result<Self, anyhow::Error> {
        if !store_directory.exists() {
            return Err(IaError::Message(format!("The file does not exist: `{:?}`", store_directory)))?;
        }
        let store = sled::open(store_directory)?;
        let tree = store.open_tree(TREE_SETTINGS)?;
        let version = tree.get("version")?.ok_or_else(||IaError::Message("The store is corupted (no version found)".to_string()))?;
        let version_str = String::from_utf8(Vec::from(version.as_ref()))?;
        if version_str.as_str() != VERSION {
            return Err(IaError::Message(format!("The file is not compatible with this version of ia : StoreVer=`{}` CurrentVer=`{}`", version_str, VERSION)))?;
        }
        Ok(Self {
            db: store,
            opened_guys: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    pub async fn get_guy_handle(&self, name: &str) -> IaResult<GuyHandle> {
        match self.opened_guys.write().map_err(|_| IaError::StoreDeadLock("Self::opened_guys"))?.entry(name.to_string()) {
            hash_map::Entry::Occupied(e) => Ok(e.get().clone()),
            hash_map::Entry::Vacant(e) => {
                let tree = self.db.open_tree(name)?;
                let handle = GuyHandle::load_or_create(tree).await?;
                e.insert(handle.clone());
                Ok(handle)
            }
        }
    }
}

impl GuyHandle {
    pub async fn load_or_create(tree: sled::Tree) -> IaResult<Self> {
        let guy: Guy = if let Some(bytes) = tree.get("template")? {
            bincode::deserialize(&bytes).unwrap()
        } else {
            Guy::new()
        };

        let guy_encoded: Vec<u8> = bincode::serialize(&guy)?;
        tree.insert("template", &guy_encoded[..])?;
        Ok(Self {
            tree,
            guy: Arc::new(RwLock::new(guy)),
        })
    }

    pub fn get_guy(&self) -> IaResult<Guy> {
        let guy = self.guy.write().map_err(|_| IaError::StoreDeadLock("GuyHandle::guy"))?.clone();
        Ok(guy)
    }

    pub fn store_guy(&self, guy: Guy) -> IaResult<()> {
        let guy_encoded: Vec<u8> = bincode::serialize(&guy)?;
        self.tree.insert("template", &guy_encoded[..])?;
        let mut guy_lock = self.guy.write().unwrap();
        let guy_lock_ref: &mut Guy = &mut guy_lock;
        let _ = std::mem::replace(guy_lock_ref, guy);
        Ok(())
    }

}