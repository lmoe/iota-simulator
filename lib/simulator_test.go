package lib

import (
	"fmt"
	"testing"
)
import "github.com/stretchr/testify/require"

func TestSimulator(t *testing.T) {
	sim, err := NewSimulator()
	require.NoError(t, err)

	checkpont, err := sim.GetLatestCheckpoint()
	require.NoError(t, err)

	fmt.Print(checkpont)
}
