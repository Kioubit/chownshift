# ChownShift

## Shift UIDs,GIDs of directories or files by a specified amount

### Features
* Supports hardlinks, symlinks
* Preserves file permissions by default (including setuid)

### Example

    # ls -la
    total 8
    drwxr-xr-x 2 user user 4096 Oct  9 15:08 .
    drwxr-xr-x 3 user user 4096 Oct  9 15:16 ..
    -rw-r--r-- 1 2000 2000    0 Oct  9 15:08 testfile
    drwxr-xr-x 1 2000 2000    0 Oct  9 15:08 testdirectory
    # chownshift -1 . # Shift UIDs and GIDs by -1
    # ls -la
    total 8
    drwxr-xr-x 2 user user 4096 Oct  9 15:08 .
    drwxr-xr-x 3 user user 4096 Oct  9 15:16 ..
    -rw-r--r-- 1 1999 1999    0 Oct  9 15:08 testfile
    drwxr-xr-x 1 1999 1999    0 Oct  9 15:08 testdirectory

### Usage
    Usage: ./chownshift <difference> <path> [Optional Arguments]
    Mandatory Arguments:
    <difference>: A positive or negative integer denoting the amount of UIDs and GIDs the files in path are to be shifted by
    <path>: The target path to a file or directory
    Optional Arguments:
    --verbose Verbose output
    --nopermissions Do not preserve permissions
    --help Show this message
    Exit codes: 0 - Success, 1 - Error, 2 - Argument error