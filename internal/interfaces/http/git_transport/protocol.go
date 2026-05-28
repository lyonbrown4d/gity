package gittransport

import (
	"encoding/hex"
	"fmt"
	"strings"

	collectionlist "github.com/arcgolabs/collectionx/list"
	"github.com/gofiber/fiber/v3"
)

func pktLine(data string) string {
	total := len(data) + 4
	return fmt.Sprintf("%04x%s", total, data)
}

type receivePackUpdate struct {
	OldSHA     string
	NewSHA     string
	RefName    string
	BranchName string
	Delete     bool
}

// ParseReceivePackUpdates extracts branch updates from a git-receive-pack request body.
func ParseReceivePackUpdates(body []byte) []receivePackUpdate {
	updates := collectionlist.NewList[receivePackUpdate]()
	offset := 0
	for {
		payload, nextOffset, ok := nextPacketPayload(body, offset)
		if !ok {
			break
		}
		offset = nextOffset
		update, ok := receivePackUpdateFromPayload(payload)
		if !ok {
			continue
		}
		updates.Add(update)
	}
	return updates.Values()
}

func nextPacketPayload(body []byte, offset int) (string, int, bool) {
	if offset+4 > len(body) {
		return "", offset, false
	}
	rawLength := string(body[offset : offset+4])
	offset += 4
	if rawLength == "0000" {
		return "", offset, false
	}
	lengthBytes, err := hex.DecodeString(rawLength)
	if err != nil || len(lengthBytes) != 2 {
		return "", offset, false
	}
	length := int(lengthBytes[0])<<8 + int(lengthBytes[1])
	if length < 4 {
		return "", offset, false
	}
	payloadLength := length - 4
	if offset+payloadLength > len(body) {
		return "", offset, false
	}
	return string(body[offset : offset+payloadLength]), offset + payloadLength, true
}

func receivePackUpdateFromPayload(payload string) (receivePackUpdate, bool) {
	if index := strings.IndexByte(payload, 0); index >= 0 {
		payload = payload[:index]
	}
	fields := strings.Fields(payload)
	if len(fields) < 3 {
		return receivePackUpdate{}, false
	}
	refName := fields[2]
	if !strings.HasPrefix(refName, "refs/heads/") {
		return receivePackUpdate{}, false
	}
	newSHA := fields[1]
	return receivePackUpdate{
		OldSHA:     fields[0],
		NewSHA:     newSHA,
		RefName:    refName,
		BranchName: strings.TrimPrefix(refName, "refs/heads/"),
		Delete:     isZeroOID(newSHA),
	}, true
}

func isZeroOID(value string) bool {
	value = strings.TrimSpace(value)
	return value == "" || value == "0000000000000000000000000000000000000000"
}

func isGitProtocolPath(path, method string) bool {
	normalized := strings.Trim(strings.ReplaceAll(path, "\\", "/"), "/")
	if normalized == "" || !strings.Contains(normalized, ".git/") {
		return false
	}
	switch method {
	case fiber.MethodGet:
		return strings.HasSuffix(normalized, "/info/refs")
	case fiber.MethodPost:
		return strings.HasSuffix(normalized, "/git-upload-pack") || strings.HasSuffix(normalized, "/git-receive-pack")
	default:
		return false
	}
}
