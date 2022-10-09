# ChownShift

## Shift UIDs,GIDs of directories or files by a specified amount

### Features
* Optional recursion mode
* Supports hardlinks, symlinks without following them
* Preserves file modes/permissions by default (including setuid)

### Example

    # ls -la
    drwxr-xr-x 2 user user 4096 Oct  9 15:08 .
    drwxr-xr-x 3 user user 4096 Oct  9 15:16 ..
    -rw-r--r-- 1 2000 2000    0 Oct  9 15:08 testfile
    drwxr-xr-x 2 2000 2000 4096 Oct  9 15:08 testdirectory
    # chownshift -1 . --recursive # Shift UIDs and GIDs by -1
    # ls -la
    drwxr-xr-x 2 user user 4096 Oct  9 15:08 .
    drwxr-xr-x 3 user user 4096 Oct  9 15:16 ..
    -rw-r--r-- 1 1999 1999    0 Oct  9 15:08 testfile
    drwxr-xr-x 2 1999 1999 4096 Oct  9 15:08 testdirectory

### Usage

    Usage: chownshift <difference> <path> [Optional Arguments]
    Mandatory Arguments:
    <difference>: A positive or negative integer denoting the amount of UIDs and GIDs the files in path are to be shifted by
    <path>: The target path to a file or directory
    Optional Arguments:
    --recursive Recurse though the path provided
    --verbose Verbose output
    --nopermissions Do not preserve permissions
    --dry-run Simulate only
    --help Show this message
    Exit codes: 0 - Success, 1 - Error, 2 - Argument error