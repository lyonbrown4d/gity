package gittransport

import (
	"encoding/hex"
	"fmt"
	"strings"

	collectionlist "github.com/arcgolabs/collectionx/list"
	"github.com/gofiber/fiber/v2"
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

func parseReceivePackUpdates(body []byte) []receivePackUpdate {
	updates := collectionlist.NewList[receivePackUpdate]()
	offset := 0
	for offset+4 <= len(body) {
		rawLength := string(body[offset : offset+4])
		offset += 4
		if rawLength == "0000" {
			break
		}
		lengthBytes, err := hex.DecodeString(rawLength)
		if err != nil || len(lengthBytes) != 2 {
			break
		}
		length := int(lengthBytes[0])<<8 + int(lengthBytes[1])
		if length < 4 {
			break
		}
		payloadLength := length - 4
		if offset+payloadLength > len(body) {
			break
		}
		payload := string(body[offset : offset+payloadLength])
		offset += payloadLength
		if index := strings.IndexByte(payload, 0); index >= 0 {
			payload = payload[:index]
		}
		fields := strings.Fields(payload)
		if len(fields) < 3 {
			continue
		}
		refName := fields[2]
		if strings.HasPrefix(refName, "refs/heads/") {
			newSHA := fields[1]
			updates.Add(receivePackUpdate{
				OldSHA:     fields[0],
				NewSHA:     newSHA,
				RefName:    refName,
				BranchName: strings.TrimPrefix(refName, "refs/heads/"),
				Delete:     isZeroOID(newSHA),
			})
		}
	}
	return updates.Values()
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
