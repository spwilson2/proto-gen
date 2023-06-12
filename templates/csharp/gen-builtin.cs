namespace Proto {
    class Builtin {
        public bool SerializeJsonInto(byte[] arr)
        {
            string json = JsonSerializer.Serialize(this);
            var json_bytes = TypeToBytes(json);
            if (json_bytes.Length < arr.Length) {
                json_bytes.CopyTo(arr, 0);
            }
            // Would do this for the next field: arr = arr[size..];
            return true;
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
        public interface  IMessaage {

        }
    }
}
