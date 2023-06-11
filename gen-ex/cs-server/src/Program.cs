﻿using System;
using System.Diagnostics;
using System.Diagnostics.CodeAnalysis;
using System.Net;
using System.Net.Sockets;
using System.Runtime.InteropServices;
using System.Text.Json;

namespace HelloWorld
{

    class Program
    {
        static void Main(string[] args)
        {
            //var ser = new FrontendService.Server(("127.0.0.1", 11005));
            //ser.Start();
            //ser.Join();
            var e = new BackendService.InputEvent();
            e.keyCode = BackendService.KeyCode.Spacebar;
            Console.WriteLine(JsonSerializer.Serialize(e));
        }
    }


    public class FrontendService
    {
        public struct Header
        {
            public UInt64 size;
            public string type;
        }

        // Autogenerated
        [StructLayout(LayoutKind.Sequential)]
        public struct MoveCommand : Command
        {
            public UInt64 uid;
            public float pos_x;
            public float pos_y;

            public static MoveCommand deserialize(byte[] data)
            {
                var cmd = new MoveCommand();


                cmd.uid = ByteToType<UInt64>(data);
                cmd.uid = NetworkToHostOrder(cmd.uid);
                data = data[Marshal.SizeOf(cmd.uid)..];

                cmd.pos_x = ByteToType<float>(data);
                cmd.pos_x = NetworkToHostOrder(cmd.pos_x);
                data = data[Marshal.SizeOf(cmd.pos_x)..];

                cmd.pos_y = ByteToType<float>(data);
                cmd.pos_y = NetworkToHostOrder(cmd.pos_y);
                data = data[Marshal.SizeOf(cmd.pos_y)..];

                return cmd;
            }
            public const string typeid = "Move";
        }
        // Autogenerated
        [StructLayout(LayoutKind.Sequential)]
        public struct SpawnCommand: Command
        {
            public UInt32 id;
        }

        // Autogenerated
        public interface Command
        {
        }
        // Automatically generate for all supported types.
        public static UInt64 NetworkToHostOrder(UInt64 s) {
            if (BitConverter.IsLittleEndian) {
                byte[] bytes = BitConverter.GetBytes(s);
                System.Array.Reverse(bytes);
                s = ByteToType<UInt64>(bytes);
            }
            return s;
        }
        // Automatically generate for all supported types.
        public static float NetworkToHostOrder(float s) {
            if (BitConverter.IsLittleEndian) {
                byte[] bytes = BitConverter.GetBytes(s);
                System.Array.Reverse(bytes);
                s = ByteToType<float>(bytes);
            }
            return s;
        }
        // Automatically generate for all supported types.
        public static int NetworkToHostOrder(int s) {
            if (BitConverter.IsLittleEndian) {
                byte[] bytes = BitConverter.GetBytes(s);
                System.Array.Reverse(bytes);
                s = ByteToType<int>(bytes);
            }
            return s;
        }

        // Autogenerated
        public static T? ByteToType<T>(byte[] data)
        {
            var reader = new BinaryReader(new MemoryStream(data));
            byte[] bytes = reader.ReadBytes(Marshal.SizeOf(typeof(T)));

            GCHandle handle = GCHandle.Alloc(bytes, GCHandleType.Pinned);
            T? theStructure = (T?)Marshal.PtrToStructure(handle.AddrOfPinnedObject(), typeof(T));
            handle.Free();
            return theStructure;
        }

        // Autogenerate
        public static (int, Command?) ParseCommand(string type, int start, byte[] data) {
            switch (type) {
                case MoveCommand.typeid: 
                // TODO: Need to handle endianess
                MoveCommand? cmd = MoveCommand.deserialize(data[start..]);
                return (start + System.Runtime.InteropServices.Marshal.SizeOf(cmd), (Command?)cmd);

                default: 
                throw new Exception($"Unexpected type ${type}");
            }

        }

        public class Server
        {
            private Thread backgroundThread;
            private Queue<Command> queue;
            private SpinLock spinlock;
            private UdpClient? listener;
            private (string, int) sock_config;

            public Server((string, int) config) {
                queue = new Queue<Command>();
                spinlock = new SpinLock();
                sock_config = config;
                backgroundThread = new Thread(new ThreadStart(this.Run));
            }

            public void Start()
            {
                // Create a thread
                backgroundThread.Start();
            }
            public void Join() {
                backgroundThread.Join();

            }

            public bool ReadQueue([MaybeNullWhen(false)] out Command result)
            {
                bool taken = false;
                spinlock.Enter(ref taken);
                var res = queue.TryDequeue(out result);
                spinlock.Exit();
                return res;
            }
            private void PushQueue(Command data)
            {
                bool taken = false;
                spinlock.Enter(ref taken);
                queue.Enqueue(data);
                spinlock.Exit();
            }

            public void Run()
            {
                while (true)
                {
                    try
                    {
                        _RunLoop();
                    }
                    //catch (SocketException e) {}
                    //catch (Exception e) { }
                    finally
                    {
                        if (listener != null) {
                            listener.Close();
                            listener = null;
                        }
                    }
                }
            }
            public void _RunLoop() {
                var (ip, port) = sock_config;
                listener = new UdpClient(port);
                while (true)
                {
                    // For each received packet, parse and push all contained messages.
                    IPEndPoint groupEP = new IPEndPoint(IPAddress.Any, 0);
                    var data = listener.Receive(ref groupEP);
                    var start = 0;
                    var cont = true;
                    while (cont) {
                        (start, cont)  = ParseMessage(start, data);
                    }
                }
            }

            private (int, Header?) ParseHeader(int start, byte[] data) {
                var count_header_len = Marshal.SizeOf(typeof(UInt64));
                if (data.Length < count_header_len + start) {
                    return (start, null);
                }
                Header header = new Header();
                Debug.Assert(Marshal.SizeOf(header.size) == count_header_len); // Assert they are the same type.

                header.size = ByteToType<UInt64>(data[..count_header_len]);
                header.size = NetworkToHostOrder(header.size);
                if (data.Length  < (int)header.size) {
                    return (start, null);
                }
                start = count_header_len+(int)header.size;
                header.type = System.Text.Encoding.ASCII.GetString(data[count_header_len..start]);
                return (start, header);
            }

            private (int, bool) ParseMessage(int start, byte[] data) {
                // Parse Header
                (start, var header) = ParseHeader(start, data);
                if (header == null) {
                    return (start, false);
                }
                // Parse message based on header type
                (start, Command? cmd) = ParseCommand(header.Value.type, start, data);
                if (cmd == null) {
                    return (start, false);
                }
                PushQueue(cmd);
                return (start, true);
            }

        }
    }
}