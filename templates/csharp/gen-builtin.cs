using System;
using System.Diagnostics;
using System.Diagnostics.CodeAnalysis;
using System.Net;
using System.Net.Sockets;
using System.Runtime.InteropServices;
using System.Text;
using System.Text.Json;

namespace Proto {
    public class Builtin {
        public static int? SerializeJsonInto<T>(T obj, byte[] bytes) {
            var s = JsonSerializer.Serialize(obj);
            var b = Encoding.UTF8.GetBytes(s);
            if (b.Length > bytes.Length) {
                return null;
            }
            b.ToArray().CopyTo(bytes, 0);
            return b.Length;
        }

        public static byte[] TypeToBytes<T>(T obj)
        {
            // https://stackoverflow.com/questions/3278827/how-to-convert-a-structure-to-a-byte-array-in-c
            int size = Marshal.SizeOf(obj);
            byte[] arr = new byte[size];

            IntPtr ptr = IntPtr.Zero;
            try
            {
                ptr = Marshal.AllocHGlobal(size);
                Marshal.StructureToPtr(obj, ptr, true);
                Marshal.Copy(ptr, arr, 0, size);
            }
            finally
            {
                Marshal.FreeHGlobal(ptr);
            }
            return arr;
        }

        public static int? findStructJsonBounds(byte[] bytes) {
            System.Collections.IEnumerator iter = bytes.GetEnumerator();
            if (!iter.MoveNext())  {
                return null;
            }
            if ((byte)iter.Current != '{')  {
                return null;
            }
            var count = 1;
            var indent = 1;
            while (iter.MoveNext()) {
                count += 1;
                // TODO/FIXME: Need to support detecting if inside a string.
                if ((byte)iter.Current == '{')  {
                    indent += 1;
                }
                if ((byte)iter.Current == '}')  {
                    indent -= 1;
                    if (indent == 0) {
                        return count;
                    }
                }
            }
            return null;
        }
        public struct RpcHeader : IMessage {
            public string msg_id {get;set;}

            public static (RpcHeader?, int) tryDeserialize(byte[] bytes) {
                var bound = Builtin.findStructJsonBounds(bytes);
                if (bound == null) {
                    return (null, 0);
                }
                return (JsonSerializer.Deserialize<RpcHeader>(bytes[..bound.Value]), bound.Value);
            }

            public int? serializeInto(byte[] bytes) {
                return Builtin.SerializeJsonInto(this, bytes);
            }
        }

        }
        public interface  IMessage {

            public int? serializeInto(byte[] bytes);
        }

}
