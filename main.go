package main

import (
	"fmt"
	"os"
	"path/filepath"
	"runtime"
	"strconv"
	"strings"
	"syscall"

	"golang.org/x/sys/unix"
)

var verbose = false
var noPermissions = false
var recursive = false
var dryRun = false

func main() {

	if runtime.GOOS != "linux" {
		fmt.Fprintln(os.Stderr, "WARNING: This program has only been tested under linux")
	}

	// -----------------------------------------------
	// Parse commandline arguments
	var cmdArgs []string = make([]string, 0)
	for _, arg := range os.Args {
		if strings.HasPrefix(arg, "--") {
			arg = strings.TrimPrefix(arg, "--")
			switch arg {
			case "verbose":
				verbose = true
			case "recursive":
				recursive = true
			case "nopermissions":
				noPermissions = true
			case "dry-run":
				dryRun = true
				verbose = true
			default:
				printUsage()
				return
			}
		} else {
			cmdArgs = append(cmdArgs, arg)
		}
	}

	if len(cmdArgs) != 3 {
		printUsage()
		os.Exit(2)
	}
	// -----------------------------------------------

	target := cmdArgs[2]

	var err error
	difference, err := strconv.Atoi(cmdArgs[1])
	if err != nil {
		fmt.Fprintln(os.Stderr, "Error: Invalid integer for difference value")
		printUsage()
		os.Exit(2)
	}

	verboseLog("Simulating...")
	if recursive {
		err = applyRecursively(target, difference, true)
	} else {
		knownInodes := make([]uint64, 0)
		var info os.FileInfo
		info, err = os.Lstat(target)
		if err != nil {
			fmt.Fprintln(os.Stderr, "Error: "+err.Error())
			os.Exit(1)
		}
		err = applyToFile(&knownInodes, target, info, difference, true)
	}
	if err != nil {
		fmt.Fprintln(os.Stderr, "Error: "+err.Error())
		os.Exit(1)
	}

	if !dryRun {
		verboseLog("Applying...")
		if recursive {
			err = applyRecursively(target, difference, false)
		} else {
			knownInodes := make([]uint64, 0)
			var info os.FileInfo
			info, err = os.Lstat(target)
			if err != nil {
				fmt.Fprintln(os.Stderr, "Error: "+err.Error())
				os.Exit(1)
			}
			err = applyToFile(&knownInodes, target, info, difference, false)
		}
		if err != nil {
			fmt.Fprintln(os.Stderr, "Error: "+err.Error())
			os.Exit(1)
		}
	}
}

func applyRecursively(target string, difference int, simulate bool) error {
	knownInodes := make([]uint64, 0)
	return filepath.Walk(target, func(path string, info os.FileInfo, err error) error {
		if err != nil {
			return err
		}

		if path == "." && target != "." {
			return nil
		}

		return applyToFile(&knownInodes, path, info, difference, simulate)
	})
}

func applyToFile(knownInodes *[]uint64, path string, info os.FileInfo, difference int, simulate bool) error {
	stat := info.Sys().(*syscall.Stat_t)
	if stat == nil {
		return fmt.Errorf("could not perform stat on %s", path)
	}

	nlink := stat.Nlink
	inode := stat.Ino
	if nlink > 1 {
		found := false
		for _, b := range *knownInodes {
			if b == inode {
				found = true
				break
			}
		}
		if !found {
			*knownInodes = append(*knownInodes, inode)
		} else {
			verboseLog(fmt.Sprintf("[Simulating] %s: Skip hard link ", path))
			return nil
		}
	}

	startgid := stat.Gid
	startuid := stat.Uid

	rawgid := int64(startgid) + int64(difference)
	rawuid := int64(startuid) + int64(difference)
	if rawgid < 0 || rawuid < 0 {
		return fmt.Errorf("%s: trying to assing uid or gid less than zero. (%d,%d -> %d,%d)", path, startuid, startgid, rawuid, rawgid)
	}

	var targetgid, targetuid uint32
	targetgid = uint32(rawgid)
	targetuid = uint32(rawuid)

	if simulate {
		verboseLog(fmt.Sprintf("[Simulating] %s: %d,%d -> %d,%d", path, startuid, startgid, targetuid, targetgid))
	} else {
		verboseLog(fmt.Sprintf("[Changing] %s: %d,%d -> %d,%d", path, startuid, startgid, targetuid, targetgid))
	}

	if !simulate {
		err := os.Lchown(path, int(targetuid), int(targetgid))
		if err != nil {
			return fmt.Errorf("chown %s: %s", path, err.Error())
		}

		if !noPermissions {
			//Restore permissions (important for setuid that gets lost after chown)
			if info.Mode()&os.ModeSymlink != os.ModeSymlink {
				//In case this is not a symlink (Cannot set permissions on symlink under linux)
				err := unix.Fchmodat(unix.AT_FDCWD, path, stat.Mode, 0)
				if err != nil {
					err = &os.PathError{Op: "chmod", Path: path, Err: err}
					return err
				}
			}
		}

	}
	return nil

}

func verboseLog(log string) {
	if verbose {
		fmt.Println(log)
	}
}

func printUsage() {
	fmt.Println("Usage: " + os.Args[0] + " <difference> <path> [Optional Arguments]")
	fmt.Println("Mandatory Arguments:")
	fmt.Println("<difference>: A positive or negative integer denoting the amount of UIDs and GIDs the files in path are to be shifted by")
	fmt.Println("<path>: The target path to a file or directory")
	fmt.Println("Optional Arguments:")
	fmt.Println("--recursive Recurse though the path provided")
	fmt.Println("--verbose Verbose output")
	fmt.Println("--nopermissions Do not preserve permissions")
	fmt.Println("--dry-run Simulate only")
	fmt.Println("--help Show this message")
	fmt.Println("Exit codes: 0 - Success, 1 - Error, 2 - Argument error")
}
