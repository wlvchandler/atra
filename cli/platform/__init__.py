import os
from . import platforms

__all__ = [
    'get_venv_paths',
    'configure_venv',
    'setup_executable',
    'print_next_steps',
]

_platform_module_to_load = None

# populate the current namespace according to platform
if os.name == platforms._WINDOWS:
    from .env import windows as _platform_module_to_load
elif os.name == platforms._POSIX:
    from .env import unix as _platform_module_to_load

if _platform_module_to_load:
    for _function_name in __all__:
        try:
            globals()[_function_name] = getattr(_platform_module_to_load, _function_name)
        except AttributeError:
            def _missing_function_factory(name, module_name):
                def _missing_function(*args, **kwargs):
                    raise AttributeError(
                        f"atra-cli.func_not_impl  func:{name} module:{module_name}  os:{os.name}  comment:'func defined in __all__ is not implemented'"
                    )
                return _missing_function
            globals()[_function_name] = _missing_function_factory(_function_name, _platform_module_to_load.__name__)
else:
    def _unsupported_os_factory(name):
        def _unsupported_os_function(*args, **kwargs):
            raise NotImplementedError(
                f"atra-cli.func_unsupported func:{name} os:{os.name}"
            )
        return _unsupported_os_function

    for _function_name in __all__:
        globals()[_function_name] = _unsupported_os_factory(_function_name)
