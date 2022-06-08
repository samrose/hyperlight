using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Runtime.InteropServices;
using System.Text;
using System.Threading.Tasks;
using Hyperlight.Core;
using Xunit;
using Xunit.Abstractions;

namespace Hyperlight.Tests
{


    public class SandboxHostTest
    {
        private readonly ITestOutputHelper output;
        public const int NUMBER_OF_ITERATIONS = 10;
        public const int NUMBER_OF_PARALLEL_TESTS = 100;
        public SandboxHostTest(ITestOutputHelper output)
        {
            //TODO: implment skip for this
            Assert.True(Sandbox.IsSupportedPlatform, "Hyperlight Sandbox is not supported on this platform.");
            this.output = output;
        }

        public class TestData
        {
            public enum ExposeMembersToGuest
            {
                Instance,
                Type,
                TypeAndNull,
                TypeAndInstance,
                InstanceAndNull,
                Null,
                All
            }
            public int ExpectedReturnValue;
            public string ExpectedOutput;
            public string GuestBinaryPath { get; }
            public Type? instanceOrTypeType { get; }
            public int NumberOfIterations { get; }
            public int NumberOfParallelTests { get; }

            public ExposeMembersToGuest ExposeMembers = ExposeMembersToGuest.All;

            public TestData(string guestBinaryFileName, string expectedOutput = "Hello, World!!\n", int? expectedReturnValue = null, int numberOfIterations = 0, int numberOfParallelTests = 0, ExposeMembersToGuest exposeMembers = ExposeMembersToGuest.All)
            {
                var path = AppDomain.CurrentDomain.BaseDirectory;
                var guestBinaryPath = Path.Combine(path, guestBinaryFileName);
                Assert.True(File.Exists(guestBinaryPath), $"Cannot find file {guestBinaryPath} to load into hyperlight");
                this.GuestBinaryPath = guestBinaryPath;
                this.ExpectedOutput = expectedOutput;
                this.ExpectedReturnValue = expectedReturnValue ?? expectedOutput.Length;
                NumberOfIterations = numberOfIterations > 0 ? numberOfIterations : NUMBER_OF_ITERATIONS;
                NumberOfParallelTests = numberOfParallelTests > 0 ? numberOfParallelTests : NUMBER_OF_PARALLEL_TESTS;
                this.ExposeMembers = exposeMembers;
            }

            public TestData(Type instanceOrTypeType, string guestBinaryFileName, string expectedOutput = "Hello, World!!\n", int? expectedReturnValue = null, int numberOfIterations = 0, int numberOfParallelTests = 0, ExposeMembersToGuest exposeMembers = ExposeMembersToGuest.All) : this(guestBinaryFileName, expectedOutput, expectedReturnValue, numberOfIterations, numberOfParallelTests, exposeMembers)
            {
                this.instanceOrTypeType = instanceOrTypeType;
            }

            public object?[] TestInstanceOrTypes()
            {
                if (instanceOrTypeType != null)
                {
                    return ExposeMembers switch
                    {
                        ExposeMembersToGuest.Instance => new object?[] { GetInstance(instanceOrTypeType) },
                        ExposeMembersToGuest.Type => new object?[] { instanceOrTypeType },
                        ExposeMembersToGuest.TypeAndNull => new object?[] { instanceOrTypeType, null },
                        ExposeMembersToGuest.TypeAndInstance => new object?[] { instanceOrTypeType, GetInstance(instanceOrTypeType) },
                        ExposeMembersToGuest.InstanceAndNull => new object?[] { GetInstance(instanceOrTypeType), null },
                        ExposeMembersToGuest.Null => new object?[] { null },
                        ExposeMembersToGuest.All => new object?[] { instanceOrTypeType, GetInstance(instanceOrTypeType), null },
                        _ => Array.Empty<object?>(),
                    };
                }

                return Array.Empty<object?>();
            }
        }
        public class SimpleTestMembers
        {
            public Func<String, int>? PrintOutput;
        }

        public class CallbackTestMembers
        {
            public Func<String, int>? PrintOutput;
            public Func<String, int>? GuestMethod;
            public Func<String, int>? GuestMethod1;
            public int HostMethod(string msg)
            {
                return PrintOutput!($"Host Method Received: {msg} from Guest");
            }
            public int HostMethod1(string msg)
            {
                return PrintOutput!($"Host Method 1 Received: {msg} from Guest");
            }
        }

        public class NoExposedMembers { }
        public class ExposedMembers
        {
            public int GetOne() => 1;
            public static int GetTwo() => 2;
            public int MethodWithArgs(string arg1, int arg2) { return arg1.Length + arg2; }
            public static int StaticMethodWithArgs(string arg1, int arg2) { return arg1.Length + arg2; }
            public int HostMethod1(string msg)
            {
                return PrintOutput!($"Host Method 1 Received: {msg} from Guest");
            }
            public Func<String, int>? GuestMethod1 = null;
            public Func<String, int>? PrintOutput = null;
        }

        [ExposeToGuest(false)]
        public class ExposeStaticMethodsUsingAttribute
        {
            public void GetNothing() { }

            public static void StaticGetNothing() { }
            [ExposeToGuest(true)]
            public static int StaticGetInt() { return 10; }
            public void GetNothingWithArgs(string arg1, int arg2) { }
            public static void StaticGetNothingWithArgs(string arg1, int arg2) { }
            public static Func<string, int>? GuestMethod = null;
            [ExposeToGuest(true)]
            public static int HostMethod1(string msg)
            {
                return msg.Length;
            }
        }

        [ExposeToGuest(false)]
        public class ExposeInstanceMethodsUsingAttribute
        {
            public void GetNothing() { }
            public static void StaticGetNothing() { }
            public void GetNothingWithArgs(string arg1, int arg2) { }
            public static void StaticGetNothingWithArgs(string arg1, int arg2) { }
            [ExposeToGuest(true)]
            public Func<string, int>? GuestMethod = null;
            [ExposeToGuest(true)]
            public int HostMethod(string msg)
            {
                return PrintOutput!($"Host Received: {msg} from Guest");
            }
            [ExposeToGuest(true)]
            public int HostMethod1(string msg)
            {
                return PrintOutput!($"Host Method 1 Received: {msg} from Guest");
            }
            [ExposeToGuest(true)]
            public Func<String, int>? PrintOutput = null;
        }

        public class DontExposeSomeMembersUsingAttribute
        {
            public int GetOne() => 1;
            public static int GetTwo() => 2;
            public int MethodWithArgs(string arg1, int arg2) { return arg1.Length + arg2; }
            public static int StaticMethodWithArgs(string arg1, int arg2) { return arg1.Length + arg2; }
            [ExposeToGuest(false)]
            public void GetNothing() { }
            [ExposeToGuest(false)]
            public static void StaticGetNothing() { }
            [ExposeToGuest(false)]
            public void GetNothingWithArgs(string arg1, int arg2) { }
            [ExposeToGuest(false)]
            public static void StaticGetNothingWithArgs(string arg1, int arg2) { }
        }

        class GuestFunctionErrors
        {
            public Func<string, int>? FunctionDoesntExist = null;
            public Func<int>? PrintOutput = null;
            public Func<int, int>? GuestMethod = null;
        }

        public class StackOverflowTests
        {
            public Func<int, int>? StackAllocate;
            public Func<int, int>? StackOverflow;
            public Func<int, int>? LargeVar;
            public Func<int, int>? SmallVar;
        }

        public class BufferOverrunTests
        {
            public Func<string, int>? BufferOverrun;
        }

        [FactSkipIfNotWindowsAndNoHypervisor]
        public void Test_Heap_Size()
        {
            var options = GetSandboxRunOptions();
            var path = AppDomain.CurrentDomain.BaseDirectory;
            var guestBinaryFileName = "simpleguest.exe";
            var guestBinaryPath = Path.Combine(path, guestBinaryFileName);
            // Heap size is set on the assembly metadata and linker arguments using the build property GUESTHEAPSIZE
            // this is set in \src\tests\Directory.Build.props
            // the value used can be changed by running msbuild with /p:GUESTHEAPSIZE=VALUE
            var heapSize = GetAssemblyMetadataAttribute("GUESTHEAPSIZE");
           
            using (var sandbox = new Sandbox(new SandboxMemoryConfiguration(), guestBinaryPath, options[0]))
            {
                CheckSize(heapSize, sandbox, "GetHeapSizeAddress");
            }
        }

        public int GetAssemblyMetadataAttribute(string name)
        {
            var value = GetType().Assembly.GetCustomAttributes<AssemblyMetadataAttribute>().Where(a=>a.Key==name).Select(a=>a.Value).FirstOrDefault();
            Assert.NotNull(value);
            Assert.True(int.TryParse(value, out int intValue));
            return intValue;
        }


        [FactSkipIfNotWindowsAndNoHypervisor]
        public void Test_Stack_Size()
        {
            var options = GetSandboxRunOptions();
            if (options.Length == 0)
            {
                return;
            }

            var path = AppDomain.CurrentDomain.BaseDirectory;
            var guestBinaryFileName = "simpleguest.exe";
            var guestBinaryPath = Path.Combine(path, guestBinaryFileName);
            // Stack size is set on the assembly metadata and linker arguments using the build property GUESTSTACKSIZE
            // this is set in \src\tests\Directory.Build.props
            // the value used can be changed by running msbuild with /p:GUESTSTACKSIZE=VALUE
            var stackSize = GetAssemblyMetadataAttribute("GUESTSTACKSIZE");
           
            using (var sandbox = new Sandbox(new SandboxMemoryConfiguration(), guestBinaryPath, options[0]))
            {
                var bindingFlags = BindingFlags.NonPublic | BindingFlags.Instance;
                var fieldInfo = sandbox.GetType().GetField("sandboxMemoryManager", bindingFlags);
                Assert.NotNull(fieldInfo);
                var sandboxMemoryManager = fieldInfo!.GetValue(sandbox);
                Assert.NotNull(sandboxMemoryManager);
                fieldInfo = sandboxMemoryManager!.GetType().GetField("sandboxMemoryLayout", bindingFlags);
                Assert.NotNull(fieldInfo);
                var sandboxMemoryLayout = fieldInfo!.GetValue(sandboxMemoryManager);
                Assert.NotNull(sandboxMemoryLayout);
                fieldInfo = sandboxMemoryLayout!.GetType().GetField("stackSize", bindingFlags);
                Assert.NotNull(fieldInfo);
                long configuredStackSize = (long)fieldInfo!.GetValue(sandboxMemoryLayout);
                Assert.Equal(stackSize, configuredStackSize);
            }
        }

        [FactSkipIfNotWindows]
        public void Test_Memory_Size_From_GuestBinary()
        {
            var option = SandboxRunOptions.RunFromGuestBinary;
            var path = AppDomain.CurrentDomain.BaseDirectory;
            var guestBinaryFileName = "simpleguest.exe";
            var guestBinaryPath = Path.Combine(path, guestBinaryFileName);
            var sandboxMemoryConfiguration = new SandboxMemoryConfiguration();

            ulong expectedSize = GetExpectedMemorySize(sandboxMemoryConfiguration, guestBinaryPath, option);

            using (var sandbox = new Sandbox(sandboxMemoryConfiguration, guestBinaryPath, option))
            {
                var size = GetMemorySize(sandbox);
                Assert.Equal(expectedSize, size);
            }

        }
        [FactSkipIfNotWindows]
        public void Test_Memory_Size_InProcess()
        {
            var path = AppDomain.CurrentDomain.BaseDirectory;
            var guestBinaryFileName = "simpleguest.exe";
            var guestBinaryPath = Path.Combine(path, guestBinaryFileName);
            var sandboxMemoryConfiguration = new SandboxMemoryConfiguration();
            var options = new SandboxRunOptions[] { SandboxRunOptions.RunInProcess, SandboxRunOptions.RunInProcess | SandboxRunOptions.RecycleAfterRun };

            foreach (var option in options)
            {
                ulong expectedSize = GetExpectedMemorySize(sandboxMemoryConfiguration, guestBinaryPath, option);
                using (var sandbox = new Sandbox(sandboxMemoryConfiguration, guestBinaryPath, option))
                {
                    var size = GetMemorySize(sandbox);
                    Assert.Equal(expectedSize, size);
                }
            }
        }
        [Fact]
        public void Test_Buffer_Overrun()
        {
            var path = AppDomain.CurrentDomain.BaseDirectory;
            var guestBinaryFileName = "simpleguest.exe";
            var guestBinaryPath = Path.Combine(path, guestBinaryFileName);
            var sandboxMemoryConfiguration = new SandboxMemoryConfiguration();
            var options = GetSandboxRunOptions();
            foreach (var option in options)
            {
                using (var sandbox = new Sandbox(GetSandboxMemoryConfiguration(), guestBinaryPath, option))
                {
                    var functions = new BufferOverrunTests();
                    sandbox.BindGuestFunction("BufferOverrun", functions);
                    var ex = Record.Exception(() =>
                    {
                        string arg = "This is a test and it should cause a GS_CHECK_FAILED error";
                        functions.BufferOverrun!(arg);
                    });
                    Assert.NotNull(ex);
                    Assert.IsType<Hyperlight.Core.HyperlightException>(ex);
                    Assert.Equal("GS_CHECK_FAILED:", ex.Message);
                }
                using (var sandbox = new Sandbox(GetSandboxMemoryConfiguration(), guestBinaryPath, option))
                {
                    var functions = new BufferOverrunTests();
                    sandbox.BindGuestFunction("BufferOverrun", functions);
                    var result = 0;
                    var ex = Record.Exception(() =>
                    {
                        string arg = "This should work!";
                        result = functions.BufferOverrun!(arg);
                    });
                    Assert.Null(ex);

                    Assert.Equal(0, result);
                }
            }
        }

        [FactSkipIfHypervisorNotPresent]
        public void Test_Stack_Overflow()
        {
            var path = AppDomain.CurrentDomain.BaseDirectory;
            var guestBinaryFileName = "simpleguest.exe";
            var guestBinaryPath = Path.Combine(path, guestBinaryFileName);
            var sandboxMemoryConfiguration = new SandboxMemoryConfiguration();
            var options = GetSandboxRunInHyperVisorOptions(); 
            var size = GetAssemblyMetadataAttribute("GUESTSTACKSIZE") / 2;

            // StackOverflow(int) function allocates a 16384 sized array and recursively calls itself int times
            
            var shouldNotOverflow = (GetAssemblyMetadataAttribute("GUESTSTACKSIZE") / 16384) - 2;
            var shouldOverflow = shouldNotOverflow * 2;

            foreach (var option in options)
            {
                using (var sandbox = new Sandbox(GetSandboxMemoryConfiguration(), guestBinaryPath, option))
                {
                    var functions = new StackOverflowTests();
                    sandbox.BindGuestFunction("StackAllocate", functions);
                    var ex = Record.Exception(() =>
                    {
                        int arg = 0;
                        functions.StackAllocate!(arg);
                    });
                    Assert.NotNull(ex);
                    Assert.IsType<System.StackOverflowException>(ex);
                    Assert.Equal("Operation caused a stack overflow.", ex.Message);
                }
                using (var sandbox = new Sandbox(GetSandboxMemoryConfiguration(), guestBinaryPath, option))
                {
                    var functions = new StackOverflowTests();
                    sandbox.BindGuestFunction("StackAllocate", functions);
                    var result = functions.StackAllocate!(size);
                    Assert.Equal(size, result);
                }
                using (var sandbox = new Sandbox(GetSandboxMemoryConfiguration(), guestBinaryPath, option))
                {
                    var functions = new StackOverflowTests();
                    sandbox.BindGuestFunction("StackOverflow", functions);
                    var ex = Record.Exception(() =>
                    {
                        var result = functions.StackOverflow!(shouldOverflow);
                    });
                    Assert.NotNull(ex);
                    Assert.IsType<System.StackOverflowException>(ex);
                    Assert.Equal("Operation caused a stack overflow.", ex.Message);
                }
                using (var sandbox = new Sandbox(GetSandboxMemoryConfiguration(), guestBinaryPath, option))
                {
                    var functions = new StackOverflowTests();
                    sandbox.BindGuestFunction("StackOverflow", functions);
                    var result = functions.StackOverflow!(shouldNotOverflow);
                    Assert.Equal(-1, result);
                }

                // This test should overwrite the entire guest memory and cause a MMIO that should get detected as a stack overflow.

                using (var sandbox = new Sandbox(GetSandboxMemoryConfiguration(), guestBinaryPath, option))
                {
                    var memorySize = GetMemorySize(sandbox);
                    var iterations = (int)(memorySize / 16384) * 2;
                    var functions = new StackOverflowTests();
                    sandbox.BindGuestFunction("StackOverflow", functions);
                    var ex = Record.Exception(() =>
                    {
                        var result = functions.StackOverflow!(iterations);
                    });
                    Assert.NotNull(ex);
                    Assert.IsType<System.StackOverflowException>(ex);
                    Assert.Equal("Operation caused a stack overflow.", ex.Message);
                }
                
                using (var sandbox = new Sandbox(GetSandboxMemoryConfiguration(), guestBinaryPath, option))
                {
                    var functions = new StackOverflowTests();
                    sandbox.BindGuestFunction("LargeVar", functions);
                    var ex = Record.Exception(() =>
                    {
                        var result = functions.LargeVar!(0);
                    });
                    Assert.NotNull(ex);
                    Assert.IsType<System.StackOverflowException>(ex);
                    Assert.Equal("Operation caused a stack overflow.", ex.Message);
                }
                using (var sandbox = new Sandbox(GetSandboxMemoryConfiguration(), guestBinaryPath, option))
                {
                    var functions = new StackOverflowTests();
                    sandbox.BindGuestFunction("SmallVar", functions);
                    var result = functions.SmallVar!(0);
                    Assert.Equal(1024, result);
                }
            }
        }

        [FactSkipIfHypervisorNotPresent]
        public void Test_Memory_Size_InHypervisor()
        {
            var path = AppDomain.CurrentDomain.BaseDirectory;
            var guestBinaryFileName = "simpleguest.exe";
            var guestBinaryPath = Path.Combine(path, guestBinaryFileName);
            var sandboxMemoryConfiguration = new SandboxMemoryConfiguration();
            var options = new SandboxRunOptions[] { SandboxRunOptions.None, SandboxRunOptions.RecycleAfterRun, SandboxRunOptions.None | SandboxRunOptions.RecycleAfterRun };

            foreach (var option in options)
            {
                ulong expectedSize = GetExpectedMemorySize(sandboxMemoryConfiguration, guestBinaryPath, option);
                using (var sandbox = new Sandbox(sandboxMemoryConfiguration, guestBinaryPath, option))
                {
                    var size = GetMemorySize(sandbox);
                    Assert.Equal(expectedSize, size);
                }
            }
        }

        [Fact]
        public void Test_Maximum_Memory_Size()
        {
            var option = SandboxRunOptions.RunInProcess;
            var path = AppDomain.CurrentDomain.BaseDirectory;
            var guestBinaryFileName = "simpleguest.exe";
            var guestBinaryPath = Path.Combine(path, guestBinaryFileName);
            var sandboxMemoryConfiguration = new SandboxMemoryConfiguration(inputDataSize: 1073741824);
            var ex = Record.Exception(() =>
            {
                using var sandbox = new Sandbox(sandboxMemoryConfiguration, guestBinaryPath, option);
                
            });
            Assert.NotNull(ex);
            Assert.IsType<ArgumentException>(ex);
            Assert.StartsWith("Total memory size ", ex.Message);
        }

        private ulong GetExpectedMemorySize(SandboxMemoryConfiguration sandboxMemoryConfiguration, string guestBinaryPath, SandboxRunOptions option)
        {

            // Heap size and Stack size are set on the assembly metadata and linker arguments using the build property GUESTHEAPSIZE and GUESTSTACKSIZE
            // this is set in \src\tests\Directory.Build.props
            // the value used can be changed by running msbuild with /p:GUESTHEAPSIZE=VALUE
            var heapSize = GetAssemblyMetadataAttribute("GUESTHEAPSIZE");
           
            // the value used can be changed by running msbuild with /p:GUESTSTACKSIZE=VALUE
            var stackSize = GetAssemblyMetadataAttribute("GUESTSTACKSIZE");

            long pageTableSize = 0x3000;
            var headerSize = 120;
            ulong totalSize;
            var codeSize = 0;
            if (!option.HasFlag(SandboxRunOptions.RunFromGuestBinary))
            {
                var codePayload = File.ReadAllBytes(guestBinaryPath);
                codeSize = codePayload.Length;
            }
            totalSize = (ulong)(codeSize
                                + stackSize
                                + heapSize
                                + pageTableSize
                                + sandboxMemoryConfiguration.HostFunctionDefinitionSize
                                + sandboxMemoryConfiguration.InputDataSize
                                + sandboxMemoryConfiguration.OutputDataSize
                                + sandboxMemoryConfiguration.HostExceptionSize
                                + sandboxMemoryConfiguration.GuestErrorMessageSize
                                + headerSize);

            var rem = totalSize % 4096;
            if (rem != 0)
            {
                totalSize += 4096 - rem;
            }
            return totalSize;
        }

        private static ulong GetMemorySize(Sandbox sandbox)
        {
            var bindingFlags = BindingFlags.NonPublic | BindingFlags.Instance;
            var fieldInfo = sandbox.GetType().GetField("sandboxMemoryManager", bindingFlags);
            Assert.NotNull(fieldInfo);
            var sandboxMemoryManager = fieldInfo!.GetValue(sandbox);
            Assert.NotNull(sandboxMemoryManager);
            fieldInfo = sandboxMemoryManager!.GetType().GetField("sandboxMemoryLayout", bindingFlags);
            Assert.NotNull(fieldInfo);
            var sandboxMemoryLayout = fieldInfo!.GetValue(sandboxMemoryManager);
            Assert.NotNull(sandboxMemoryLayout);
            var methodInfo = sandboxMemoryLayout!.GetType().GetMethod("GetMemorySize", bindingFlags);
            Assert.NotNull(methodInfo);
            var size = methodInfo!.Invoke(sandboxMemoryLayout, Array.Empty<object>());
            Assert.NotNull(size);
            return (ulong)size;
        }

        [Fact]
        public void Test_Guest_Error_Message_Size()
        {
            var options = GetSandboxRunOptions();
            var path = AppDomain.CurrentDomain.BaseDirectory;
            var guestBinaryFileName = "simpleguest.exe";
            var guestBinaryPath = Path.Combine(path, guestBinaryFileName);

            foreach (var option in options)
            {
                var size = GetErrorMessageSize();
                using (var sandbox = new Sandbox(new SandboxMemoryConfiguration(guestErrorMessageSize: size), guestBinaryPath, option))
                {
                    CheckSize(size, sandbox, "GetGuestErrorMessageSizeAddress");
                }
            }
        }

        [Fact]
        public void Test_Function_Definitions_Size()
        {
            var options = GetSandboxRunOptions();
            var path = AppDomain.CurrentDomain.BaseDirectory;
            var guestBinaryFileName = "simpleguest.exe";
            var guestBinaryPath = Path.Combine(path, guestBinaryFileName);

            foreach (var option in options)
            {
                var size = GetFunctionDefinitionSize();
                using (var sandbox = new Sandbox(new SandboxMemoryConfiguration(functionDefinitionSize: size), guestBinaryPath, option))
                {
                    CheckSize(size, sandbox, "GetFunctionDefinitionSizeAddress");
                }
            }
        }

        [Fact]
        public void Test_InputData_Size()
        {
            var options = GetSandboxRunOptions();
            var path = AppDomain.CurrentDomain.BaseDirectory;
            var guestBinaryFileName = "simpleguest.exe";
            var guestBinaryPath = Path.Combine(path, guestBinaryFileName);

            foreach (var option in options)
            {
                var size = GetInputDataSize();
                using (var sandbox = new Sandbox(new SandboxMemoryConfiguration(inputDataSize: size), guestBinaryPath, option))
                {
                    CheckSize(size, sandbox, "GetInputDataSizeAddress");
                }
            }
        }

        [Fact]
        public void Test_OutputData_Size()
        {
            var options = GetSandboxRunOptions();
            var path = AppDomain.CurrentDomain.BaseDirectory;
            var guestBinaryFileName = "simpleguest.exe";
            var guestBinaryPath = Path.Combine(path, guestBinaryFileName);

            foreach (var option in options)
            {
                var size = GetOutputDataSize();
                using (var sandbox = new Sandbox(new SandboxMemoryConfiguration(outputDataSize: size), guestBinaryPath, option))
                {
                    CheckSize(size, sandbox, "GetOutputDataSizeAddress");
                }
            }
        }

        [Fact]
        public void Test_Config_Minimum_Sizes()
        {
            var minInputSize = 0x2000;
            var minOutputSize = 0x2000;
            var minHostFunctionDefinitionSize = 0x400;
            var minHostExceptionSize = 0x400;
            var minGuestErrorMessageSize = 0x80;
            var sandboxMemoryConfiguration = new SandboxMemoryConfiguration(0, 0, 0, 0, 0);
            Assert.Equal(minInputSize, sandboxMemoryConfiguration.InputDataSize);
            Assert.Equal(minOutputSize, sandboxMemoryConfiguration.OutputDataSize);
            Assert.Equal(minHostFunctionDefinitionSize, sandboxMemoryConfiguration.HostFunctionDefinitionSize);
            Assert.Equal(minHostExceptionSize, sandboxMemoryConfiguration.HostExceptionSize);
            Assert.Equal(minGuestErrorMessageSize, sandboxMemoryConfiguration.GuestErrorMessageSize);
        }

        [Fact]
        public void Test_Host_Exceptions_Size()
        {
            var options = GetSandboxRunOptions();
            var path = AppDomain.CurrentDomain.BaseDirectory;
            var guestBinaryFileName = "simpleguest.exe";
            var guestBinaryPath = Path.Combine(path, guestBinaryFileName);

            foreach (var option in options)
            {
                var size = GetHostExceptionSize();
                using (var sandbox = new Sandbox(new SandboxMemoryConfiguration(hostExceptionSize: size), guestBinaryPath, option))
                {
                    CheckSize(size, sandbox, "GetHostExceptionSizeAddress");
                }
            }
        }

        private static void CheckSize(int size, Sandbox sandbox, string methodName)
        {
            var bindingFlags = BindingFlags.NonPublic | BindingFlags.Instance | BindingFlags.Public;
            var fieldInfo = sandbox.GetType().GetField("sandboxMemoryManager", bindingFlags);
            Assert.NotNull(fieldInfo);
            var sandboxMemoryManager = fieldInfo!.GetValue(sandbox);
            Assert.NotNull(sandboxMemoryManager);
            fieldInfo = sandboxMemoryManager!.GetType().GetField("sandboxMemoryLayout", bindingFlags);
            Assert.NotNull(fieldInfo);
            var sandboxMemoryLayout = fieldInfo!.GetValue(sandboxMemoryManager);
            Assert.NotNull(sandboxMemoryLayout);
            var propInfo = sandboxMemoryManager!.GetType().GetProperty("SourceAddress", bindingFlags);
            Assert.NotNull(propInfo);
            var sourceAddress = propInfo!.GetValue(sandboxMemoryManager);
            Assert.NotNull(sourceAddress);
            var methodInfo = sandboxMemoryLayout!.GetType().GetMethod(methodName, bindingFlags);
            Assert.NotNull(methodInfo);
            var addr = methodInfo!.Invoke(sandboxMemoryLayout, new object[] { sourceAddress! });
            Assert.NotNull(addr);
            Assert.IsType<IntPtr>(addr);
            var hostExceptionSize = Marshal.ReadInt64((IntPtr)addr!);
            Assert.Equal(size, hostExceptionSize);
        }

        [Fact]
        public void Test_Invalid_Guest_Function_Causes_Exception()
        {
            var options = GetSandboxRunOptions();
            var path = AppDomain.CurrentDomain.BaseDirectory;
            var guestBinaryFileName = "callbackguest.exe";
            var guestBinaryPath = Path.Combine(path, guestBinaryFileName);

            foreach (var option in options)
            {
                using (var sandbox = new Sandbox(GetSandboxMemoryConfiguration(), guestBinaryPath, option))
                {
                    var functions = new GuestFunctionErrors();
                    sandbox.BindGuestFunction("FunctionDoesntExist", functions);
                    var ex = Record.Exception(() =>
                    {
                        string arg = string.Empty;
                        functions.FunctionDoesntExist!(arg);
                    });
                    Assert.NotNull(ex);
                    Assert.IsType<HyperlightException>(ex);
                    Assert.Equal("GUEST_FUNCTION_NOT_FOUND:FunctionDoesntExist", ex.Message);
                }
            }
        }

        [Fact]
        public void Test_Invalid_Type_Of_Guest_Function_Parameter_Causes_Exception()
        {
            var options = GetSandboxRunOptions();
            var path = AppDomain.CurrentDomain.BaseDirectory;
            var guestBinaryFileName = "callbackguest.exe";
            var guestBinaryPath = Path.Combine(path, guestBinaryFileName);

            foreach (var option in options)
            {
                using (var sandbox = new Sandbox(GetSandboxMemoryConfiguration(), guestBinaryPath, option))
                {
                    var functions = new GuestFunctionErrors();
                    sandbox.BindGuestFunction("GuestMethod", functions);
                    var ex = Record.Exception(() =>
                    {
                        functions.GuestMethod!(1);
                    });
                    Assert.NotNull(ex);
                    Assert.IsType<HyperlightException>(ex);
                    Assert.Equal("UNSUPPORTED_PARAMETER_TYPE:0", ex.Message);
                }
            }
        }

        [Fact]
        public void Test_Invalid_Number_Of_Guest_Function_Parameters_Causes_Exception()
        {
            var options = GetSandboxRunOptions();
            var path = AppDomain.CurrentDomain.BaseDirectory;
            var guestBinaryFileName = "callbackguest.exe";
            var guestBinaryPath = Path.Combine(path, guestBinaryFileName);

            foreach (var option in options)
            {
                using (var sandbox = new Sandbox(GetSandboxMemoryConfiguration(), guestBinaryPath, option))
                {
                    var functions = new GuestFunctionErrors();
                    sandbox.BindGuestFunction("PrintOutput", functions);
                    var ex = Record.Exception(() =>
                    {
                        functions.PrintOutput!();
                    });
                    Assert.NotNull(ex);
                    Assert.IsType<HyperlightException>(ex);
                    Assert.Equal("GUEST_FUNCTION_PARAMETERS_MISSING:", ex.Message);
                }
            }
        }

        private SandboxRunOptions[] GetSandboxRunOptions()
        {
            if (RuntimeInformation.IsOSPlatform(OSPlatform.Windows))
            {
                if (Sandbox.IsHypervisorPresent())
                {
                   return new SandboxRunOptions[] { SandboxRunOptions.RunFromGuestBinary, SandboxRunOptions.None, SandboxRunOptions.RunInProcess, SandboxRunOptions.RunInProcess | SandboxRunOptions.RecycleAfterRun, SandboxRunOptions.RecycleAfterRun, SandboxRunOptions.None | SandboxRunOptions.RecycleAfterRun };
                }
                else
                {
                   return new SandboxRunOptions[] { SandboxRunOptions.RunFromGuestBinary, SandboxRunOptions.RunInProcess, SandboxRunOptions.RunInProcess | SandboxRunOptions.RecycleAfterRun };
                }
            }
            return GetSandboxRunInHyperVisorOptions();
        }

        private SandboxRunOptions[] GetSandboxRunInHyperVisorOptions()
        {
            if (Sandbox.IsHypervisorPresent())
            {
                 return new SandboxRunOptions[] { SandboxRunOptions.None, SandboxRunOptions.RecycleAfterRun, SandboxRunOptions.None | SandboxRunOptions.RecycleAfterRun };
            }
            return new SandboxRunOptions[] { };
        }

        [Fact]
        public void Test_Bind_And_Expose_Methods()
        {
            var options = GetSandboxRunOptions();
            var path = AppDomain.CurrentDomain.BaseDirectory;
            var guestBinaryFileName = "simpleguest.exe";
            var guestBinaryPath = Path.Combine(path, guestBinaryFileName);
            Assert.True(File.Exists(guestBinaryPath), $"Cannot find file {guestBinaryPath} to load into hyperlight");

            List<(Type type, List<string> exposedMethods, List<string> boundDelegates, List<string> exposedStaticMethods)> testData = new()
            {
                (typeof(NoExposedMembers), new(), new(), new()),
                (typeof(ExposedMembers), new() { "GetOne", "MethodWithArgs", "HostMethod1" }, new() { "GuestMethod1", "PrintOutput" }, new() { "GetTwo", "StaticMethodWithArgs" }),
                (typeof(ExposeStaticMethodsUsingAttribute), new(), new(), new() { "StaticGetInt", "HostMethod1" }),
                (typeof(ExposeInstanceMethodsUsingAttribute), new() { "HostMethod", "HostMethod1" }, new() { "GuestMethod" }, new()),
                (typeof(DontExposeSomeMembersUsingAttribute), new() { "GetOne", "MethodWithArgs" }, new(), new() { "GetTwo", "StaticMethodWithArgs" }),
            };
            foreach (var option in options)
            {
                foreach (var (type, exposedMethods, boundDelegates, exposedStaticMethods) in testData)
                {
                    foreach (var target in new object?[] { type, GetInstance(type) })
                    {
                        using (var sandbox = new Sandbox(GetSandboxMemoryConfiguration(), guestBinaryPath, option))
                        {
                            if (target is Type)
                            {
                                sandbox.ExposeHostMethods(target as Type);
                            }
                            else
                            {
                                sandbox.ExposeAndBindMembers(target);
                            }
                            Assert.NotNull(sandbox);
                            List<string> methodNames = target is Type ? exposedStaticMethods : exposedMethods.Concat(exposedStaticMethods).ToList();
                            CheckExposedMethods(sandbox, methodNames, target);
                            if (target is not Type)
                            {
                                CheckBoundDelegates(sandbox, target!, boundDelegates);
                            }
                        }
                    }
                }

                var delegateNames = new List<string> { "GuestMethod", "GuestMethod1" };
                var hostMethods = new List<string> { "HostMethod", "HostMethod1" };
                using (var sandbox = new Sandbox(GetSandboxMemoryConfiguration(), guestBinaryPath, option))
                {
                    var instance = new CallbackTestMembers();
                    foreach (var delegateName in delegateNames)
                    {
                        sandbox.BindGuestFunction(delegateName, instance);
                    }
                    foreach (var hostMethod in hostMethods)
                    {
                        sandbox.ExposeHostMethod(hostMethod, instance);
                    }
                    CheckBoundDelegates(sandbox, instance!, delegateNames);
                    CheckExposedMethods(sandbox, hostMethods, instance);
                }
            }
        }

        private void CheckBoundDelegates(Sandbox sandbox, object target, List<string> delegateNames)
        {
            foreach (var d in delegateNames!)
            {
                var fieldInfo = target.GetType().GetField(d, BindingFlags.Public | BindingFlags.Instance);
                Assert.NotNull(fieldInfo);
                Assert.True(typeof(Delegate).IsAssignableFrom(fieldInfo!.FieldType));
                Assert.NotNull(fieldInfo.GetValue(target));
            }
        }

        private void CheckExposedMethods(Sandbox sandbox, List<string> methodNames, object target)
        {
            var bindingFlags = BindingFlags.Public | BindingFlags.NonPublic | BindingFlags.Instance;
            var fieldInfo = sandbox.GetType().GetField("hyperlightPEB", bindingFlags);
            Assert.NotNull(fieldInfo);
            var hyperlightPEB = fieldInfo!.GetValue(sandbox);
            Assert.NotNull(hyperlightPEB);
            fieldInfo = hyperlightPEB!.GetType().GetField("listFunctions", bindingFlags);
            Assert.NotNull(fieldInfo);
            var listFunctions = fieldInfo!.GetValue(hyperlightPEB) as IEnumerable<object>;
            Assert.NotNull(listFunctions);
            Assert.Equal(methodNames.Count, listFunctions!.Count());

            foreach (var f in listFunctions!)
            {
                var propertyInfo = f.GetType().GetProperty("FunctionName", bindingFlags);
                Assert.NotNull(propertyInfo);
                Assert.Contains(propertyInfo!.GetValue(f), methodNames);
            }
        }

        [Fact]
        private void Test_SandboxInit()
        {
            var options = GetSandboxRunOptions();
            var path = AppDomain.CurrentDomain.BaseDirectory;
            var guestBinaryFileName = "callbackguest.exe";
            var guestBinaryPath = Path.Combine(path, guestBinaryFileName);
            Assert.True(File.Exists(guestBinaryPath), $"Cannot find file {guestBinaryPath} to load into hyperlight");

            List<(Type type, List<string> exposedMethods, List<string> boundDelegates, List<string> exposedStaticMethods, List<(string delegateName, int returnValue, string expectedOutput, object[]? args)> expectedResults)> testData = new()
            {
                (typeof(ExposedMembers), new() { "GetOne", "MethodWithArgs", "HostMethod1" }, new() { "GuestMethod1", "PrintOutput" }, new() { "GetTwo", "StaticMethodWithArgs" }, new()
                {
                    ("GuestMethod1", 77, "Host Method 1 Received: Hello from GuestFunction1, Hello from Init from Guest", new object[] { "Hello from Init" }),
                    ("PrintOutput", 15, "Hello from Init", new object[] { "Hello from Init" }),
                }),
                (typeof(ExposeStaticMethodsUsingAttribute), new(), new(), new() { "StaticGetInt", "HostMethod1" }, new()),
                (typeof(ExposeInstanceMethodsUsingAttribute), new() { "HostMethod", "HostMethod1" }, new() { "GuestMethod", "PrintOutput" }, new(), new()
                {
                    ("GuestMethod", 67, "Host Received: Hello from GuestFunction, Hello from Init from Guest", new object[] { "Hello from Init" }),
                    ("PrintOutput", 21, "Hello again from Init", new object[] { "Hello again from Init" }),
                }),
                (typeof(DontExposeSomeMembersUsingAttribute), new() { "GetOne", "MethodWithArgs" }, new(), new() { "GetTwo", "StaticMethodWithArgs" }, new()),
            };

            Action<ISandboxRegistration> func;
            StringWriter output;
            Sandbox sandbox;

            foreach (var option in options)
            {
                foreach (var (type, exposedMethods, boundDelegates, exposedStaticMethods, expectedResults) in testData)
                {
                    output = new StringWriter();
                    var numberOfCalls = 0;
                    foreach (var target in new object[] { type, GetInstance(type) })
                    {
                        // Call init explicity creating delegates and binding methods
                        if (target is Type)
                        {
                            func = (s) =>
                            {
                                foreach (var method in exposedStaticMethods)
                                {
                                    s.ExposeHostMethod(method, (Type)target);
                                }
                            };
                        }
                        else
                        {
                            func = (s) =>
                            {
                                foreach (var method in exposedMethods)
                                {
                                    s.ExposeHostMethod(method, target);
                                }
                                foreach (var method in exposedStaticMethods)
                                {
                                    s.ExposeHostMethod(method, target.GetType());
                                }
                                foreach (var boundDelegate in boundDelegates)
                                {
                                    s.BindGuestFunction(boundDelegate, target);
                                }
                                foreach (var expectedResult in expectedResults)
                                {
                                    InvokeMethod(target, expectedResult.delegateName, expectedResult.returnValue, expectedResult.expectedOutput, expectedResult.args, output, ref numberOfCalls);
                                }
                            };
                        }

                        sandbox = new Sandbox(GetSandboxMemoryConfiguration(), guestBinaryPath, option, func, output);
                        if (target is Type)
                        {
                            CheckExposedMethods(sandbox, exposedStaticMethods, (Type)target);
                        }
                        else
                        {
                            Assert.Equal<int>(expectedResults.Count, numberOfCalls);
                        }
                        sandbox.Dispose();
                    }

                    foreach (var target in new object[] { type, GetInstance(type) })
                    {

                        // Pass instance/type via constructor and call methods in init.

                        if (target is Type)
                        {
                            func = null;
                        }
                        else
                        {
                            func = (s) =>
                            {
                                s.ExposeAndBindMembers(target);
                                foreach (var expectedResult in expectedResults)
                                {
                                    InvokeMethod(target, expectedResult.delegateName, expectedResult.returnValue, expectedResult.expectedOutput, expectedResult.args, output, ref numberOfCalls);
                                }
                            };
                        }

                        numberOfCalls = 0;
                        output = new StringWriter();
                        sandbox = new Sandbox(GetSandboxMemoryConfiguration(), guestBinaryPath, option, func, output);
                        if (target is Type)
                        {
                            sandbox.ExposeHostMethods(target as Type);
                            CheckExposedMethods(sandbox, exposedStaticMethods, (Type)target);
                        }
                        else
                        {
                            Assert.Equal<int>(expectedResults.Count, numberOfCalls);
                        }
                        sandbox.Dispose();

                    }
                }
            }
        }

        private void InvokeMethod(object target, string delgateName, int returnValue, string expectedOutput, object[]? args, StringWriter output, ref int numberOfCalls)
        {
            numberOfCalls++;
            var del = GetDelegate(target, delgateName);
            var result = del.DynamicInvoke(args);
            Assert.Equal(returnValue, (int)result);
            var builder = output.GetStringBuilder();
            Assert.Equal(expectedOutput, builder.ToString());
            builder.Remove(0, builder.Length);
        }

        private static Delegate GetDelegate(object target, string delegateName)
        {
            var fieldInfo = target.GetType().GetField(delegateName, BindingFlags.Public | BindingFlags.Instance);
            Assert.NotNull(fieldInfo);
            Assert.True(typeof(Delegate).IsAssignableFrom(fieldInfo.FieldType));
            var fieldValue = fieldInfo.GetValue(target);
            Assert.NotNull(fieldValue);
            var del = (Delegate)fieldValue!;
            Assert.NotNull(del);
            return del;
        }

        [TheorySkipIfNotWindows]
        [MemberData(nameof(GetSimpleTestData))]
        public void Test_Loads_Windows_Exe(TestData testData)
        {
            SandboxRunOptions[] options = { SandboxRunOptions.RunFromGuestBinary, SandboxRunOptions.RunFromGuestBinary | SandboxRunOptions.RunInProcess };
            foreach (var option in options)
            {
                RunTests(testData, option, SimpleTest);
            }
        }

        [TheorySkipIfNotWindows]
        [MemberData(nameof(GetSimpleTestData))]
        public void Test_Loads_Windows_Exe_Concurrently(TestData testData)
        {
            SandboxRunOptions[] options = { SandboxRunOptions.RunFromGuestBinary, SandboxRunOptions.RunFromGuestBinary | SandboxRunOptions.RunInProcess };
            foreach (var option in options)
            {
                var output1 = new StringWriter();
                var sandbox1 = new Sandbox(GetSandboxMemoryConfiguration(), testData.GuestBinaryPath, option, output1);
                var builder = output1.GetStringBuilder();
                SimpleTest(sandbox1, testData, output1, builder);
                var output2 = new StringWriter();
                Sandbox sandbox2 = null;
                var ex = Record.Exception(() => sandbox2 = new Sandbox(GetSandboxMemoryConfiguration(), testData.GuestBinaryPath, option, output2));
                Assert.NotNull(ex);
                Assert.IsType<System.ApplicationException>(ex);
                Assert.Equal("Only one instance of Sandbox is allowed when running from guest binary", ex.Message);
                sandbox1.Dispose();
                sandbox2?.Dispose();
                output1.Dispose();
                output2.Dispose();
                using (var output = new StringWriter())
                {
                    using (var sandbox = new Sandbox(GetSandboxMemoryConfiguration(), testData.GuestBinaryPath, option, output))
                    {
                        builder = output.GetStringBuilder();
                        SimpleTest(sandbox, testData, output, builder);
                    }
                }
            }
        }

        [FactSkipIfNotLinux]
        public void Test_Throws_InProcess_On_Linux()
        {
            SandboxRunOptions[] options = { SandboxRunOptions.RunInProcess, SandboxRunOptions.RunInProcess | SandboxRunOptions.RecycleAfterRun };
            foreach (var option in options)
            {
                var binary = "simpleguest.exe";
                var path = AppDomain.CurrentDomain.BaseDirectory;
                var guestBinaryPath = Path.Combine(path, binary);
                var ex = Record.Exception(() =>
                {
                    var sandbox = new Sandbox(GetSandboxMemoryConfiguration(), guestBinaryPath, option);
                    var guestMethods = new SimpleTestMembers();
                    sandbox.BindGuestFunction("PrintOutput", guestMethods);
                    var result = guestMethods.PrintOutput("This will throw an exception");
                });
                Assert.NotNull(ex);
                Assert.IsType<NotSupportedException>(ex);
                Assert.Equal("Cannot run in process on Linux", ex.Message);
            }
        }

        [TheorySkipIfNotWindows]
        [MemberData(nameof(GetSimpleTestData))]
        public void Test_Runs_InProcess(TestData testData)
        {
            SandboxRunOptions[] options = { SandboxRunOptions.RunInProcess, SandboxRunOptions.RunInProcess | SandboxRunOptions.RecycleAfterRun };
            foreach (var option in options)
            {
                RunTests(testData, option, SimpleTest);
            }
        }

        [TheorySkipIfNotWindows]
        [MemberData(nameof(GetSimpleTestData))]
        public void Test_Runs_InProcess_Concurrently(TestData testData)
        {
            Parallel.For(0, testData.NumberOfParallelTests, (t) =>
            {
                SandboxRunOptions[] options = { SandboxRunOptions.RunInProcess, SandboxRunOptions.RunInProcess | SandboxRunOptions.RecycleAfterRun };
                foreach (var option in options)
                {
                    RunTests(testData, option, SimpleTest);
                }
            });
        }

        [TheorySkipIfHyperVisorNotPresent]
        [MemberData(nameof(GetSimpleTestData))]
        public void Test_Runs_InHyperVisor(TestData testData)
        {
            SandboxRunOptions[] options = { SandboxRunOptions.None, SandboxRunOptions.None | SandboxRunOptions.RecycleAfterRun };
            foreach (var option in options)
            {
                RunTests(testData, option, SimpleTest);
            }

            // Run tests using constructors without options
            foreach (var instanceOrType in testData.TestInstanceOrTypes())
            {
                RunTest(testData, instanceOrType, SimpleTest);
            }
        }

        [TheorySkipIfHyperVisorNotPresent]
        [MemberData(nameof(GetSimpleTestData))]
        public void Test_Runs_InHyperVisor_Concurrently(TestData testData)
        {

            Parallel.For(0, testData.NumberOfParallelTests, (t) =>
            {
                SandboxRunOptions[] options = { SandboxRunOptions.None, SandboxRunOptions.None | SandboxRunOptions.RecycleAfterRun };
                foreach (var option in options)
                {
                    RunTests(testData, option, SimpleTest);
                }

                // Run tests using constructors without options
                foreach (var instanceOrType in testData.TestInstanceOrTypes())
                {
                    RunTest(testData, instanceOrType, SimpleTest);
                }
            });

        }

        [TheorySkipIfNotWindows]
        [MemberData(nameof(GetCallbackTestData))]
        public void Test_Loads_Windows_Exe_With_Callback(TestData testData)
        {
            SandboxRunOptions[] options = { SandboxRunOptions.RunFromGuestBinary, SandboxRunOptions.RunFromGuestBinary | SandboxRunOptions.RunInProcess };
            foreach (var option in options)
            {
                RunTests(testData, option, CallbackTest);
            }
        }

        [TheorySkipIfNotWindows]
        [MemberData(nameof(GetCallbackTestData))]
        public void Test_Loads_Windows_Exe_With_Callback_Concurrently(TestData testData)
        {
            SandboxRunOptions[] options = { SandboxRunOptions.RunFromGuestBinary, SandboxRunOptions.RunFromGuestBinary | SandboxRunOptions.RunInProcess };
            foreach (var option in options)
            {
                foreach (var instanceOrType in testData.TestInstanceOrTypes())
                {
                    var output1 = new StringWriter();
                    var builder = output1.GetStringBuilder();
                    var sandbox1 = new Sandbox(GetSandboxMemoryConfiguration(), testData.GuestBinaryPath, option, null, output1);
                    if (instanceOrType is not null)
                    {
                        if (instanceOrType is Type)
                        {
                            sandbox1.ExposeHostMethods(instanceOrType as Type);
                        }
                        else
                        {
                            sandbox1.ExposeAndBindMembers(instanceOrType);
                        }
                    }
                    CallbackTest(sandbox1, testData, output1, builder, instanceOrType);
                    var output2 = new StringWriter();
                    Sandbox sandbox2 = null;
                    object instanceOrType1;
                    if (instanceOrType is not null && instanceOrType is not Type)
                    {
                        instanceOrType1 = GetInstance(instanceOrType.GetType());
                    }
                    else
                    {
                        instanceOrType1 = instanceOrType;
                    }
                    var ex = Record.Exception(() => sandbox2 = new Sandbox(GetSandboxMemoryConfiguration(), testData.GuestBinaryPath, option, null, output2));
                    Assert.NotNull(ex);
                    Assert.IsType<System.ApplicationException>(ex);
                    Assert.Equal("Only one instance of Sandbox is allowed when running from guest binary", ex.Message);
                    sandbox1.Dispose();
                    sandbox2?.Dispose();
                    output1.Dispose();
                    output2.Dispose();
                    using (var output = new StringWriter())
                    {
                        builder = output.GetStringBuilder();
                        using (var sandbox = new Sandbox(GetSandboxMemoryConfiguration(), testData.GuestBinaryPath, option, null, output))
                        {
                            if (instanceOrType1 is not null)
                            {
                                if (instanceOrType1 is Type)
                                {
                                    sandbox.ExposeHostMethods(instanceOrType1 as Type);
                                }
                                else
                                {
                                    sandbox.ExposeAndBindMembers(instanceOrType1);
                                }
                            }
                            CallbackTest(sandbox, testData, output, builder, instanceOrType1);
                        }
                    }
                }
            }
        }

        [TheorySkipIfNotWindows]
        [MemberData(nameof(GetCallbackTestData))]
        public void Test_Runs_InProcess_With_Callback(TestData testData)
        {
            SandboxRunOptions[] options = { SandboxRunOptions.RunInProcess, SandboxRunOptions.RunInProcess | SandboxRunOptions.RecycleAfterRun };
            foreach (var option in options)
            {
                RunTests(testData, option, CallbackTest);
            }
        }

        [TheorySkipIfNotWindows]
        [MemberData(nameof(GetCallbackTestData))]
        public void Test_Runs_InProcess_With_Callback_Concurrently(TestData testData)
        {
            Parallel.For(0, testData.NumberOfParallelTests, (t) =>
            {
                SandboxRunOptions[] options = { SandboxRunOptions.RunInProcess };
                foreach (var option in options)
                {
                    RunTests(testData, option, CallbackTest);
                }
            });
        }

        [TheorySkipIfHyperVisorNotPresent]
        [MemberData(nameof(GetCallbackTestData))]
        public void Test_Runs_InHyperVisor_With_Callback(TestData testData)
        {
            SandboxRunOptions[] options = { SandboxRunOptions.None, SandboxRunOptions.None | SandboxRunOptions.RecycleAfterRun };
            foreach (var option in options)
            {
                RunTests(testData, option, CallbackTest);
            }

            // Run tests using constructors without options
            foreach (var instanceOrType in testData.TestInstanceOrTypes())
            {
                RunTest(testData, instanceOrType, CallbackTest);
            }
        }


        [TheorySkipIfHyperVisorNotPresent]
        [MemberData(nameof(GetCallbackTestData))]
        public void Test_Runs_InHyperVisor_With_Callback_Concurrently(TestData testData)
        {
            Parallel.For(0, testData.NumberOfParallelTests, (t) =>
            {
                SandboxRunOptions[] options = { SandboxRunOptions.None, SandboxRunOptions.None | SandboxRunOptions.RecycleAfterRun };
                foreach (var option in options)
                {
                    RunTests(testData, option, CallbackTest);
                }

                // Run tests using constructors without options
                foreach (var instanceOrType in testData.TestInstanceOrTypes())
                {
                    RunTest(testData, instanceOrType, CallbackTest);
                }
            });
        }

        private void RunTests(TestData testData, SandboxRunOptions options, Action<Sandbox, TestData, StringWriter, StringBuilder, object> test)
        {
            if (testData.TestInstanceOrTypes().Length > 0)
            {
                foreach (var instanceOrType in testData.TestInstanceOrTypes())
                {
                    RunTest(testData, options, instanceOrType, test);
                }
            }
            else
            {
                RunTest(testData, options, test);
            }
        }

        private void RunTest(TestData testData, SandboxRunOptions sandboxRunOptions, object? instanceOrType, Action<Sandbox, TestData, StringWriter, StringBuilder, object> test)
        {
            using (var output = new StringWriter())
            {
                using (var sandbox = new Sandbox(GetSandboxMemoryConfiguration(), testData.GuestBinaryPath, sandboxRunOptions, null, output))
                {
                    if (instanceOrType is not null)
                    {
                        if (instanceOrType is Type)
                        {
                            sandbox.ExposeHostMethods(instanceOrType as Type);
                        }
                        else
                        {
                            sandbox.ExposeAndBindMembers(instanceOrType);
                        }
                    }
                    var numberOfIterations = sandboxRunOptions.HasFlag(SandboxRunOptions.RecycleAfterRun) ? testData.NumberOfIterations : 1;
                    var builder = output.GetStringBuilder();
                    for (var i = 0; i < numberOfIterations; i++)
                    {
                        builder.Remove(0, builder.Length);
                        test(sandbox, testData, output, builder, instanceOrType);
                    }
                }
            }
        }

        private void RunTest(TestData testData, SandboxRunOptions sandboxRunOptions, Action<Sandbox, TestData, StringWriter, StringBuilder, object?> test)
        {
            using (var output = new StringWriter())
            {
                using (var sandbox = new Sandbox(GetSandboxMemoryConfiguration(), testData.GuestBinaryPath, sandboxRunOptions, output))
                {
                    var numberOfIterations = sandboxRunOptions.HasFlag(SandboxRunOptions.RecycleAfterRun) ? testData.NumberOfIterations : 1;
                    var builder = output.GetStringBuilder();
                    for (var i = 0; i < numberOfIterations; i++)
                    {
                        builder.Remove(0, builder.Length);
                        test(sandbox, testData, output, builder, null);
                    }
                }
            }
        }

        private void RunTest(TestData testData, object? instanceOrType, Action<Sandbox, TestData, StringWriter, StringBuilder, object?> test)
        {
            using (var output = new StringWriter())
            {
                using (var sandbox = new Sandbox(GetSandboxMemoryConfiguration(), testData.GuestBinaryPath, output))
                {
                    if (instanceOrType is not null)
                    {
                        if (instanceOrType is Type)
                        {
                            sandbox.ExposeHostMethods(instanceOrType as Type);
                        }
                        else
                        {
                            sandbox.ExposeAndBindMembers(instanceOrType);
                        }
                    }
                    var builder = output.GetStringBuilder();
                    test(sandbox, testData, output, builder, instanceOrType);
                }
            }
        }

        private void SimpleTest(Sandbox sandbox, TestData testData, StringWriter output, StringBuilder builder, object? _ = null)
        {
            var guestMethods = new SimpleTestMembers();
            sandbox.BindGuestFunction("PrintOutput", guestMethods);
            var result = guestMethods.PrintOutput!(testData.ExpectedOutput);
            Assert.Equal<int>(testData.ExpectedReturnValue, result);
            Assert.Equal(testData.ExpectedOutput, builder.ToString());
        }

        private void CallbackTest(Sandbox sandbox, TestData testData, StringWriter output, StringBuilder builder, object typeorinstance)
        {

            if (typeorinstance == null)
            {
                // TODO: Enables this by handling errors correctly in Sandbox when crossing managed native bounary.
                var guestMethods = new CallbackTestMembers();
                sandbox.BindGuestFunction("GuestMethod", guestMethods);
                var ex = Record.Exception(() =>
                {
                    var result = guestMethods.GuestMethod(testData.ExpectedOutput);
                });
                Assert.NotNull(ex);
                Assert.IsType<ArgumentException>(ex);
                Assert.Equal("HostMethod, Could not find host method name.", ex.Message);
            }
            else
            {
                if (typeorinstance is Type)
                {
                    CallGuestMethod1(sandbox, testData, builder);
                }
                else
                {
                    var message = "Hello from CallbackTest";
                    var expectedMessage = string.Format(testData.ExpectedOutput, message);
                    var fieldInfo = GetDelegateFieldInfo(typeorinstance, "GuestMethod1");
                    if (fieldInfo != null)
                    {
                        var fieldValue = fieldInfo.GetValue(typeorinstance);
                        Assert.NotNull(fieldValue);
                        var del = (Delegate)fieldValue!;
                        Assert.NotNull(del);
                        var args = new object[] { message };
                        var result = sandbox.CallGuest<int>(() => { return (int)del!.DynamicInvoke(args); });
                        Assert.Equal<int>(expectedMessage.Length, (int)result!);
                        Assert.Equal(expectedMessage, builder.ToString());
                    }
                    else
                    {
                        CallGuestMethod1(sandbox, testData, builder);
                    }
                }
            }

            static void CallGuestMethod1(Sandbox sandbox, TestData testData, StringBuilder builder)
            {
                var guestMethods = new CallbackTestMembers();
                sandbox.BindGuestFunction("GuestMethod1", guestMethods);
                sandbox.BindGuestFunction("PrintOutput", guestMethods);
                sandbox.ExposeHostMethod("HostMethod1", guestMethods);
                var message = "Hello from CallbackTest";
                var expectedMessage = string.Format(testData.ExpectedOutput, message);
                var result = sandbox.CallGuest<int>(() => { return guestMethods.GuestMethod1!(message); });
                Assert.Equal<int>(testData.ExpectedReturnValue, result);
                Assert.Equal(expectedMessage, builder.ToString());
            }
        }

        private FieldInfo? GetDelegateFieldInfo(object target, string MethodName)
        {
            var fieldInfo = target.GetType().GetField(MethodName, BindingFlags.Public | BindingFlags.Instance);
            if (fieldInfo is not null && typeof(Delegate).IsAssignableFrom(fieldInfo.FieldType))
            {
                return fieldInfo;
            }
            return null;
        }

        public static IEnumerable<object[]> GetSimpleTestData()
        {
            return new List<object[]>
            {
                new object[] { new TestData(typeof(NoExposedMembers), "simpleguest.exe") },
                new object[] { new TestData(typeof(ExposedMembers), "simpleguest.exe") },
                new object[] { new TestData(typeof(ExposeStaticMethodsUsingAttribute), "simpleguest.exe") },
                new object[] { new TestData(typeof(ExposeInstanceMethodsUsingAttribute), "simpleguest.exe") },
                new object[] { new TestData(typeof(DontExposeSomeMembersUsingAttribute), "simpleguest.exe") },
                new object[] { new TestData("simpleguest.exe") } ,
            };
        }

        public static IEnumerable<object[]> GetCallbackTestData()
        {
            return new List<object[]>
            {
                new object[] { new TestData(typeof(ExposedMembers), "callbackguest.exe", "Host Method 1 Received: Hello from GuestFunction1, {0} from Guest", 85,0,0,TestData.ExposeMembersToGuest.All) },
                new object[] { new TestData("callbackguest.exe", "Host Method 1 Received: Hello from GuestFunction1, {0} from Guest", 85, 0,0,TestData.ExposeMembersToGuest.Null) },
                new object[] { new TestData(typeof(ExposeStaticMethodsUsingAttribute), "callbackguest.exe", "Host Method 1 Received: Hello from GuestFunction1, {0} from Guest", 85,0,0, TestData.ExposeMembersToGuest.TypeAndNull) },
                new object[] { new TestData(typeof(ExposeInstanceMethodsUsingAttribute), "callbackguest.exe", "Host Method 1 Received: Hello from GuestFunction1, Hello from CallbackTest from Guest", 85, 0,0,TestData.ExposeMembersToGuest.InstanceAndNull) },
            };
        }

        private static object GetInstance(Type type) => Activator.CreateInstance(type);
        private static T GetInstance<T>() => Activator.CreateInstance<T>();
        static bool RunningOnWindows => RuntimeInformation.IsOSPlatform(OSPlatform.Windows);

        private SandboxMemoryConfiguration GetSandboxMemoryConfiguration()
        {
            var errorMessageSize = GetErrorMessageSize();
            var functionDefinitionSize = GetFunctionDefinitionSize();
            var hostExceptionSize = GetHostExceptionSize();
            var inputDataSize = GetInputDataSize();
            var outputDataSize = GetOutputDataSize();

            // This is just so sometimes the defaults are used.

            if (errorMessageSize % 8 == 0 || functionDefinitionSize % 8 == 0 || hostExceptionSize % 8 == 0)
            {
                output.WriteLine("Using Default Configuration");
                return new SandboxMemoryConfiguration();
            }
            output.WriteLine($"Using Configuration: guestErrorMessageSize: {errorMessageSize} functionDefinitionSize: {functionDefinitionSize} hostExceptionSize: {hostExceptionSize} inputDataSize: {inputDataSize} outputDataSize: {outputDataSize}");
            return new SandboxMemoryConfiguration(guestErrorMessageSize: errorMessageSize, functionDefinitionSize: functionDefinitionSize, hostExceptionSize: hostExceptionSize, inputDataSize: inputDataSize, outputDataSize: outputDataSize);
        }

        private int GetErrorMessageSize()
        {
            var min = 256;
            var max = 1024;
            var random = new Random();
            return random.Next(min, max + 1);
        }

        private int GetFunctionDefinitionSize()
        {
            var min = 1024;
            var max = 8192;
            var random = new Random();
            return random.Next(min, max + 1);
        }

        private int GetHostExceptionSize()
        {
            var min = 1024;
            var max = 8192;
            var random = new Random();
            return random.Next(min, max + 1);
        }

        private int GetInputDataSize()
        {
            var min = 8182;
            var max = 65536;
            var random = new Random();
            return random.Next(min, max + 1);
        }

        private int GetOutputDataSize()
        {
            var min = 8182;
            var max = 65536;
            var random = new Random();
            return random.Next(min, max + 1);
        }
    }
}
