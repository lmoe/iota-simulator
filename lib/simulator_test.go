package lib

import (
	"context"
	"fmt"
	"testing"

	"github.com/iotaledger/wasp/clients"
	"github.com/iotaledger/wasp/clients/iota-go/iotaclient"

	"github.com/stretchr/testify/require"
)

func TestSimulator(t *testing.T) {
	sim, err := NewSimulator()
	require.NoError(t, err)

	checkpont, err := sim.GetLatestCheckpoint()
	require.NoError(t, err)

	var c clients.L1Client

	c.ExecuteTransactionBlock(context.Background(), iotaclient.ExecuteTransactionBlockRequest{})

	fmt.Print(checkpont)
}
