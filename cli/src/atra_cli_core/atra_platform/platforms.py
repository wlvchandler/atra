from sys import version_info

_WINDOWS = 'nt'
_POSIX = 'posix'
_JAVA = 'java'

# for sys.platform (note: use sys.platform.startswith)
AIX = 'aix'
ATHEOS = 'atheos'
CYGWIN= 'cygwin'
FREEBSD = 'freebsd'
MACOSX = 'darwin'
OPENBSD = 'openbsd'
RISCOS = 'riscos'
WINDOWS = 'win32'

ANDROID  = 'Requires python >= 3.13' if version_info.minor < 13 else 'android'
EMSCRIPTEN  = 'Requires python >= 3.11' if version_info.minor < 11 else 'emscripten' 
IOS  = 'Requires python >= 3.13' if version_info.minor < 13 else 'ios' 
WASI  = 'Requires python >= 3.13' if version_info.minor < 11 else 'wasi'

# unsure if startswith resolves as expected for these
MSYS2 = 'msys2' 
OS2 = 'os2'
