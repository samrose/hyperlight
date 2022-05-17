using System;
using System.Collections.Generic;
using System.Linq;
using System.Runtime.InteropServices;

namespace Hyperlight
{
    class HyperlightPEB
    {
        struct Header
        {
            public ulong CountFunctions;
            public ulong DispatchFunction;
        }
        struct FunctionDefinition
        {
            public ulong FunctionName;
            public ulong FunctionSignature;
            public ulong Flags;
        }

        class FunctionDetails
        {
            public string FunctionName { get; set; }
            public string FunctionSignature { get; set; }
            public ulong Flags { get; set; }

        }
        readonly HashSet<FunctionDetails> listFunctions = new();
        readonly IntPtr basePtr;
        readonly int length;
        readonly long offset;

        public HyperlightPEB(IntPtr ptr, int length, long offset)
        {
            this.basePtr = ptr;
            this.length = length;
            this.offset = offset;
        }

        // TODO: Allow overloaded functions;

        public bool FunctionExists(string functionName)
        {
            return listFunctions.Where(f => f.FunctionName == functionName).Any();
        }

        public void AddFunction(string functionName, string functionSignature, ulong flags)
        {
            // TODO: Allow virtual function names so a function a name can point to a fully qualified name. 
            if (listFunctions.Where(f => f.FunctionName == functionName).Any())
            {
                throw new ArgumentException($"functionName already exists");
            }
            listFunctions.Add(new FunctionDetails() { FunctionName = functionName, FunctionSignature = functionSignature, Flags = flags });
        }
        public void Create()
        {
            var ptr = basePtr;
            var header = new Header() { CountFunctions = (ulong)listFunctions.Count, DispatchFunction = 0 };
            WriteToMemory(ptr, header);
        }

        public void Update()
        {
            var ptr = basePtr;
            var header = Marshal.PtrToStructure<Header>(ptr);
            header.CountFunctions = (ulong)listFunctions.Count;
            WriteToMemory(ptr, header);
        }

        private void WriteToMemory(IntPtr ptr, Header header)
        {
            var headerSize = Marshal.SizeOf<Header>();
            var functionDefinitionSize = Marshal.SizeOf<FunctionDefinition>();
            var totalHeaderSize = headerSize + (int)header.CountFunctions * functionDefinitionSize;
            if (totalHeaderSize > length)
            {
                throw new Exception("Not enough memory for header structures");
            }

            var stringTable = new SimpleStringTable(IntPtr.Add(ptr, totalHeaderSize), length - totalHeaderSize, offset);
            Marshal.StructureToPtr(header, ptr, false);
            ptr += headerSize;

            foreach (var func in listFunctions)
            {
                var fd = new FunctionDefinition() { FunctionName = stringTable.AddString(func.FunctionName), FunctionSignature = stringTable.AddString(func.FunctionSignature), Flags = func.Flags };
                Marshal.StructureToPtr(fd, ptr, false);
                ptr += functionDefinitionSize;
            }
        }
    }


}
