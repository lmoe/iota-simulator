package lib

import "C"

type VerifiedCheckpoint struct {
	Data struct {
		Epoch                      int    `json:"epoch"`
		SequenceNumber             int    `json:"sequence_number"`
		NetworkTotalTransactions   int    `json:"network_total_transactions"`
		ContentDigest              string `json:"content_digest"`
		PreviousDigest             string `json:"previous_digest"`
		EpochRollingGasCostSummary struct {
			ComputationCost         string `json:"computationCost"`
			ComputationCostBurned   string `json:"computationCostBurned"`
			StorageCost             string `json:"storageCost"`
			StorageRebate           string `json:"storageRebate"`
			NonRefundableStorageFee string `json:"nonRefundableStorageFee"`
		} `json:"epoch_rolling_gas_cost_summary"`
		TimestampMs           int64       `json:"timestamp_ms"`
		CheckpointCommitments []any       `json:"checkpoint_commitments"`
		EndOfEpochData        interface{} `json:"end_of_epoch_data"`
		VersionSpecificData   []int       `json:"version_specific_data"`
	} `json:"data"`
	AuthSignature struct {
		Epoch      int    `json:"epoch"`
		Signature  string `json:"signature"`
		SignersMap []int  `json:"signers_map"`
	} `json:"auth_signature"`
}

func (s *Simulator) GetLatestCheckpoint() (VerifiedCheckpoint, error) {
	return executeCall[VerifiedCheckpoint](s, "getLatestCheckpoint", nil)
}
