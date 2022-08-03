#pragma once
#define NO_ERROR                                    0   // The function call was successful
#define CODE_HEADER_NOT_SET                         1   // The expected PE header was not found in the Guest Binary
#define UNSUPPORTED_PARAMETER_TYPE                  2   // The type of the parameter is not supported by the Guest.
#define GUEST_FUNCTION_NAME_NOT_PROVIDED            3   // The Guest function name was not provided by the host.  
#define GUEST_FUNCTION_NOT_FOUND                    4   // The function does not exist in the Guest.  
#define GUEST_FUNCTION_INCORRECT_NO_OF_PARAMETERS   5   // Incorrect number of parameters for the guest function.
#define DISPATCH_FUNCTION_POINTER_NOT_SET           6   // Host Call Dispatch Function Pointer is not present.
#define OUTB_ERROR                                  7   // Error in OutB Function
#define UNKNOWN_ERROR                               8   // The guest error is unknown.
#define STACK_OVERFLOW                              9   // Guest stack allocations caused stack overflow
#define GS_CHECK_FAILED                             10  // __security_check_cookie failed
#define TOO_MANY_GUEST_FUNCTIONS                    11  // The guest tried to register too many guest functions
#define FAILURE_IN_DLMALLOC                         12  // this error is set when dlmalloc calls ABORT (e.g. function defined in #define ABORT (dlmalloc_abort() calls setError with this errorcode)
#define MALLOC_FAILED                               13  // this error is set when malloc returns 0 bytes.
#define GUEST_FUNCTION_PARAMETER_TYPE_MISMATCH      14  // The function call parameter type was not the expected type.  
#define GUEST_ERROR                                 15  // An error occurred in the guest Guest implementation should use this along with a message when calling setError.
#define ARRAY_LENGTH_PARAM_IS_MISSING               16  // Expected a int parameter to follow a byte array