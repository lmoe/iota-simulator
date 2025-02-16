use iota_types::base_types::{EpochId, IotaAddress, ObjectID, TransactionDigest, VersionNumber};
use iota_types::committee::Committee;
use iota_types::digests::{
    ChainIdentifier, CheckpointContentsDigest, CheckpointDigest, TransactionEventsDigest,
};
use iota_types::effects::{TransactionEffects, TransactionEvents};
use iota_types::messages_checkpoint::{
    CheckpointContents, CheckpointSequenceNumber, FullCheckpointContents, VerifiedCheckpoint,
};
use iota_types::object::Object;
use iota_types::storage::{
    AccountOwnedObjectInfo, CoinInfo, DynamicFieldIndexInfo, DynamicFieldKey, ObjectStore,
    ReadStore, RestStateReader,
};
use iota_types::transaction::VerifiedTransaction;
use move_core_types::language_storage::StructTag;
use simulacrum::Simulacrum;
use std::sync::{Arc, RwLock};
pub struct SimulacrumReaderWrapper {
    pub inner: Arc<RwLock<Simulacrum>>,
}

impl ObjectStore for SimulacrumReaderWrapper {
    fn get_object(
        &self,
        object_id: &ObjectID,
    ) -> iota_types::storage::error::Result<Option<Object>> {
        self.inner.read().unwrap().get_object(object_id)
    }

    fn get_object_by_key(
        &self,
        object_id: &ObjectID,
        version: VersionNumber,
    ) -> iota_types::storage::error::Result<Option<Object>> {
        self.inner
            .read()
            .unwrap()
            .get_object_by_key(object_id, version)
    }
}

impl ReadStore for SimulacrumReaderWrapper {
    fn get_committee(
        &self,
        epoch: EpochId,
    ) -> iota_types::storage::error::Result<Option<Arc<Committee>>> {
        self.inner.read().unwrap().get_committee(epoch)
    }

    fn get_latest_checkpoint(&self) -> iota_types::storage::error::Result<VerifiedCheckpoint> {
        self.inner.read().unwrap().get_latest_checkpoint()
    }

    fn get_highest_verified_checkpoint(
        &self,
    ) -> iota_types::storage::error::Result<VerifiedCheckpoint> {
        self.inner.read().unwrap().get_highest_verified_checkpoint()
    }

    fn get_highest_synced_checkpoint(
        &self,
    ) -> iota_types::storage::error::Result<VerifiedCheckpoint> {
        self.inner.read().unwrap().get_highest_synced_checkpoint()
    }

    fn get_lowest_available_checkpoint(
        &self,
    ) -> iota_types::storage::error::Result<CheckpointSequenceNumber> {
        self.inner.read().unwrap().get_lowest_available_checkpoint()
    }

    fn get_checkpoint_by_digest(
        &self,
        digest: &CheckpointDigest,
    ) -> iota_types::storage::error::Result<Option<VerifiedCheckpoint>> {
        self.inner.read().unwrap().get_checkpoint_by_digest(digest)
    }

    fn get_checkpoint_by_sequence_number(
        &self,
        sequence_number: CheckpointSequenceNumber,
    ) -> iota_types::storage::error::Result<Option<VerifiedCheckpoint>> {
        self.inner
            .read()
            .unwrap()
            .get_checkpoint_by_sequence_number(sequence_number)
    }

    fn get_checkpoint_contents_by_digest(
        &self,
        digest: &CheckpointContentsDigest,
    ) -> iota_types::storage::error::Result<Option<CheckpointContents>> {
        self.inner
            .read()
            .unwrap()
            .get_checkpoint_contents_by_digest(digest)
    }

    fn get_checkpoint_contents_by_sequence_number(
        &self,
        sequence_number: CheckpointSequenceNumber,
    ) -> iota_types::storage::error::Result<Option<CheckpointContents>> {
        self.inner
            .read()
            .unwrap()
            .get_checkpoint_contents_by_sequence_number(sequence_number)
    }

    fn get_transaction(
        &self,
        tx_digest: &TransactionDigest,
    ) -> iota_types::storage::error::Result<Option<Arc<VerifiedTransaction>>> {
        self.inner.read().unwrap().get_transaction(tx_digest)
    }

    fn get_transaction_effects(
        &self,
        tx_digest: &TransactionDigest,
    ) -> iota_types::storage::error::Result<Option<TransactionEffects>> {
        self.inner
            .read()
            .unwrap()
            .get_transaction_effects(tx_digest)
    }

    fn get_events(
        &self,
        event_digest: &TransactionEventsDigest,
    ) -> iota_types::storage::error::Result<Option<TransactionEvents>> {
        self.inner.read().unwrap().get_events(event_digest)
    }

    fn get_full_checkpoint_contents_by_sequence_number(
        &self,
        sequence_number: CheckpointSequenceNumber,
    ) -> iota_types::storage::error::Result<Option<FullCheckpointContents>> {
        self.inner
            .read()
            .unwrap()
            .get_full_checkpoint_contents_by_sequence_number(sequence_number)
    }

    fn get_full_checkpoint_contents(
        &self,
        digest: &CheckpointContentsDigest,
    ) -> iota_types::storage::error::Result<Option<FullCheckpointContents>> {
        self.inner
            .read()
            .unwrap()
            .get_full_checkpoint_contents(digest)
    }
}

impl RestStateReader for SimulacrumReaderWrapper {
    fn get_transaction_checkpoint(
        &self,
        digest: &TransactionDigest,
    ) -> iota_types::storage::error::Result<Option<CheckpointSequenceNumber>> {
        self.inner
            .read()
            .unwrap()
            .get_transaction_checkpoint(digest)
    }

    fn get_lowest_available_checkpoint_objects(
        &self,
    ) -> iota_types::storage::error::Result<CheckpointSequenceNumber> {
        self.inner
            .read()
            .unwrap()
            .get_lowest_available_checkpoint_objects()
    }

    fn get_chain_identifier(&self) -> iota_types::storage::error::Result<ChainIdentifier> {
        self.inner.read().unwrap().get_chain_identifier()
    }

    fn account_owned_objects_info_iter(
        &self,
        owner: IotaAddress,
        cursor: Option<ObjectID>,
    ) -> iota_types::storage::error::Result<Box<dyn Iterator<Item = AccountOwnedObjectInfo> + '_>>
    {
        let guard = self.inner.read().unwrap();
        let iter = guard.account_owned_objects_info_iter(owner, cursor)?;
        let owned_iter: Vec<_> = iter.collect(); // Collect into an owned Vec
        Ok(Box::new(owned_iter.into_iter())) // Return an iterator over the owned Vec
    }

    fn dynamic_field_iter(
        &self,
        parent: ObjectID,
        cursor: Option<ObjectID>,
    ) -> iota_types::storage::error::Result<
        Box<dyn Iterator<Item = (DynamicFieldKey, DynamicFieldIndexInfo)> + '_>,
    > {
        let guard = self.inner.read().unwrap();
        let iter = guard.dynamic_field_iter(parent, cursor)?;
        let owned_iter: Vec<_> = iter.collect(); // Collect into an owned Vec
        Ok(Box::new(owned_iter.into_iter())) // Return an iterator over the owned Vec
    }

    fn get_coin_info(
        &self,
        coin_type: &StructTag,
    ) -> iota_types::storage::error::Result<Option<CoinInfo>> {
        self.inner.read().unwrap().get_coin_info(coin_type)
    }
}
