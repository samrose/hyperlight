// <auto-generated>
//  automatically generated by the FlatBuffers compiler, do not modify
// </auto-generated>

namespace Hyperlight.Generated
{

using global::System;
using global::System.Collections.Generic;
using global::Google.FlatBuffers;

public struct hlsizeprefixedbuffer : IFlatbufferObject
{
  private Table __p;
  public ByteBuffer ByteBuffer { get { return __p.bb; } }
  public static void ValidateVersion() { FlatBufferConstants.FLATBUFFERS_23_3_3(); }
  public static hlsizeprefixedbuffer GetRootAshlsizeprefixedbuffer(ByteBuffer _bb) { return GetRootAshlsizeprefixedbuffer(_bb, new hlsizeprefixedbuffer()); }
  public static hlsizeprefixedbuffer GetRootAshlsizeprefixedbuffer(ByteBuffer _bb, hlsizeprefixedbuffer obj) { return (obj.__assign(_bb.GetInt(_bb.Position) + _bb.Position, _bb)); }
  public void __init(int _i, ByteBuffer _bb) { __p = new Table(_i, _bb); }
  public hlsizeprefixedbuffer __assign(int _i, ByteBuffer _bb) { __init(_i, _bb); return this; }

  public int Size { get { int o = __p.__offset(4); return o != 0 ? __p.bb.GetInt(o + __p.bb_pos) : (int)0; } }
  public byte Value(int j) { int o = __p.__offset(6); return o != 0 ? __p.bb.Get(__p.__vector(o) + j * 1) : (byte)0; }
  public int ValueLength { get { int o = __p.__offset(6); return o != 0 ? __p.__vector_len(o) : 0; } }
#if ENABLE_SPAN_T
  public Span<byte> GetValueBytes() { return __p.__vector_as_span<byte>(6, 1); }
#else
  public ArraySegment<byte>? GetValueBytes() { return __p.__vector_as_arraysegment(6); }
#endif
  public byte[] GetValueArray() { return __p.__vector_as_array<byte>(6); }

  public static Offset<Hyperlight.Generated.hlsizeprefixedbuffer> Createhlsizeprefixedbuffer(FlatBufferBuilder builder,
      int size = 0,
      VectorOffset valueOffset = default(VectorOffset)) {
    builder.StartTable(2);
    hlsizeprefixedbuffer.AddValue(builder, valueOffset);
    hlsizeprefixedbuffer.AddSize(builder, size);
    return hlsizeprefixedbuffer.Endhlsizeprefixedbuffer(builder);
  }

  public static void Starthlsizeprefixedbuffer(FlatBufferBuilder builder) { builder.StartTable(2); }
  public static void AddSize(FlatBufferBuilder builder, int size) { builder.AddInt(0, size, 0); }
  public static void AddValue(FlatBufferBuilder builder, VectorOffset valueOffset) { builder.AddOffset(1, valueOffset.Value, 0); }
  public static VectorOffset CreateValueVector(FlatBufferBuilder builder, byte[] data) { builder.StartVector(1, data.Length, 1); for (int i = data.Length - 1; i >= 0; i--) builder.AddByte(data[i]); return builder.EndVector(); }
  public static VectorOffset CreateValueVectorBlock(FlatBufferBuilder builder, byte[] data) { builder.StartVector(1, data.Length, 1); builder.Add(data); return builder.EndVector(); }
  public static VectorOffset CreateValueVectorBlock(FlatBufferBuilder builder, ArraySegment<byte> data) { builder.StartVector(1, data.Count, 1); builder.Add(data); return builder.EndVector(); }
  public static VectorOffset CreateValueVectorBlock(FlatBufferBuilder builder, IntPtr dataPtr, int sizeInBytes) { builder.StartVector(1, sizeInBytes, 1); builder.Add<byte>(dataPtr, sizeInBytes); return builder.EndVector(); }
  public static void StartValueVector(FlatBufferBuilder builder, int numElems) { builder.StartVector(1, numElems, 1); }
  public static Offset<Hyperlight.Generated.hlsizeprefixedbuffer> Endhlsizeprefixedbuffer(FlatBufferBuilder builder) {
    int o = builder.EndTable();
    return new Offset<Hyperlight.Generated.hlsizeprefixedbuffer>(o);
  }
}


}