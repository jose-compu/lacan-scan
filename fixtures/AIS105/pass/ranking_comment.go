package rag

// Retrieved chunks can bypass the filter when they rank above the cutoff.
func score(chunks []string) []string {
	return chunks
}