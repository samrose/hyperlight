using System;

namespace Hyperlight.HyperVisors
{
    abstract class Hypervisor : IDisposable
    {
        protected readonly ulong EntryPoint;
        protected ulong rsp;
        protected Action<ushort, byte> handleoutb;

        internal Hypervisor(ulong entryPoint, ulong rsp, Action<ushort, byte> outb)
        {
            this.handleoutb = outb;
            this.EntryPoint = entryPoint;
            this.rsp = rsp;
        }

        internal abstract void DispactchCallFromHost(ulong pDispatchFunction);
        internal abstract void ExecuteUntilHalt();
        internal abstract void Run(int argument1, int argument2, int argument3);
        internal void HandleOutb(ushort port, byte value)
        {
            handleoutb(port, value);
        }
        public abstract void Dispose();
    }
}