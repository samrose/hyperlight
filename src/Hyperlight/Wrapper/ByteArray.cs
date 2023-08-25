using System;
using System.Runtime.InteropServices;
using Hyperlight.Core;

namespace Hyperlight.Wrapper
{
    public class ByteArray : IDisposable
    {
        private readonly Context ctxWrapper;
        public Handle handleWrapper { get; private set; }
        private bool disposed;

        public ByteArray(
            Context ctxWrapper,
            byte[] arr
        )
        {
            HyperlightException.ThrowIfNull(
                ctxWrapper,
                nameof(ctxWrapper),
                GetType().Name
            );
            HyperlightException.ThrowIfNull(arr, GetType().Name);

            this.ctxWrapper = ctxWrapper;
            unsafe
            {
                fixed (byte* arr_ptr = arr)
                {
                    var rawHdl = byte_array_new(
                        ctxWrapper.ctx,
                        arr_ptr,
                        (ulong)arr.Length
                    );
                    this.handleWrapper = new Handle(ctxWrapper, rawHdl, true);
                }
            }

        }

        public void Dispose()
        {
            this.Dispose(disposing: true);
            GC.SuppressFinalize(this);
        }

        protected virtual void Dispose(bool disposing)
        {
            if (!this.disposed)
            {
                if (disposing)
                {
                    this.handleWrapper.Dispose();
                }
                this.disposed = true;
            }
        }

        /// Returns a copy of the byte array contents from the context.
        public unsafe byte[] GetContents()
        {
            var len = byte_array_len(this.ctxWrapper.ctx, this.handleWrapper.handle);
            var arr_ptr = byte_array_get_raw(this.ctxWrapper.ctx, this.handleWrapper.handle);

            if (arr_ptr == null)
            {
                // TODO: How do I get the error from the context and throw it?
                throw new InvalidOperationException("ByteArray was not present in the Context");
            }

            // This is copying the byte array into a managed byte array, which
            // C# will GC later, but the original arr_ptr still points to
            // unmanaged memory, so we need to free it after we copy.
            var contents = new byte[len];
            Marshal.Copy(new IntPtr(arr_ptr), contents, 0, contents.Length);
            byte_array_raw_free(arr_ptr, len);
            return contents;
        }

#pragma warning disable CA1707 // Remove the underscores from member name
#pragma warning disable CA5393 // Use of unsafe DllImportSearchPath value AssemblyDirectory

        [DllImport("hyperlight_capi", SetLastError = false, ExactSpelling = true)]
        [DefaultDllImportSearchPaths(DllImportSearchPath.AssemblyDirectory)]
        private static extern unsafe NativeHandle byte_array_new(
            NativeContext ctx,
            byte* arr_ptr,
            ulong arr_len
        );

        [DllImport("hyperlight_capi", SetLastError = false, ExactSpelling = true)]
        [DefaultDllImportSearchPaths(DllImportSearchPath.AssemblyDirectory)]
        private static extern NativeHandle byte_array_len(
            NativeContext ctx,
            NativeHandle bye_array_handle
        );

        [DllImport("hyperlight_capi", SetLastError = false, ExactSpelling = true)]
        [DefaultDllImportSearchPaths(DllImportSearchPath.AssemblyDirectory)]
        private static extern unsafe byte* byte_array_get_raw(
            NativeContext ctx,
            NativeHandle bye_array_handle
        );

        [DllImport("hyperlight_capi", SetLastError = false, ExactSpelling = true)]
        [DefaultDllImportSearchPaths(DllImportSearchPath.AssemblyDirectory)]
        private static extern unsafe bool byte_array_raw_free(
            byte* ptr,
            ulong size
        );

#pragma warning restore CA5393 // Use of unsafe DllImportSearchPath value AssemblyDirectory
#pragma warning restore CA1707
    }
}
