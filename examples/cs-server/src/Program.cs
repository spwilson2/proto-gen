using System;
using System.Diagnostics;
using System.Diagnostics.CodeAnalysis;
using System.Net;
using System.Net.Sockets;
using System.Runtime.InteropServices;
using System.Text.Json;
using System.Threading;

namespace HelloWorld
{
    class Program
    {
        static void Main(string[] args)
        {
            var frontend_server = new FrontendServer(("127.0.0.1", 10001));
            frontend_server.Start();
            var backend_client = new  BackendClient(("127.0.0.1", 10002));
            backend_client.Start();
            frontend_server.Join();
        }
    }
    public class FrontendServer
    {
        private Thread backgroundThread;
        private Queue<Proto.IGameFrontend> queue;
        private SpinLock spinlock;
        private UdpClient? listener;
        private (string, int) sock_config;

        public FrontendServer((string, int) config) {
            queue = new Queue<Proto.IGameFrontend>();
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

        public bool ReadQueue([MaybeNullWhen(false)] out Proto.IGameFrontend result)
        {
            bool taken = false;
            spinlock.Enter(ref taken);
            var res = queue.TryDequeue(out result);
            spinlock.Exit();
            return res;
        }
        private void PushQueue(Proto.IGameFrontend data)
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
                var offset = 0;
                while (true) {
                    var (msg, cont) = Proto.GameFrontend.ParseMessage(data[offset..]);
                    offset += cont;
                    if (msg == null) {
                       break;
                    }
                    PushQueue(msg);
                }
            }
        }

    }

    public class BackendClient
    {
        private Thread backgroundThread;

        private Queue<Proto.IGameBackend> queue;
        private SpinLock spinlock;
        private Semaphore queue_sem;

        private UdpClient? listener;
        private (string, int) sock_config;

        public BackendClient((string, int) config) {
            queue = new Queue<Proto.IGameBackend>();
            spinlock = new SpinLock();
            sock_config = config;
            backgroundThread = new Thread(new ThreadStart(this.Run));
            queue_sem = new Semaphore(initialCount: 0, maximumCount: int.MaxValue);

        }

        public void Start()
        {
            // Create a thread
            backgroundThread.Start();
        }
        public void Join() {
            backgroundThread.Join();

        }

        public bool ReadQueue([MaybeNullWhen(false)] out Proto.IGameBackend result)
        {
            bool taken = false;
            result = null;
            if (!queue_sem.WaitOne(10)) {
                return false;
            }
            spinlock.Enter(ref taken);
            var res = queue.TryDequeue(out result);
            spinlock.Exit();
            return res;
        }

        public bool ReadQueueBlocking([MaybeNullWhen(false)] out Proto.IGameBackend result)
        {
            bool taken = false;
            queue_sem.WaitOne();
            spinlock.Enter(ref taken);
            var res = queue.TryDequeue(out result);
            spinlock.Exit();
            return res;
        }
        public void PushQueue(Proto.IGameBackend data)
        {
            bool taken = false;
            spinlock.Enter(ref taken);
            queue.Enqueue(data);
            queue_sem.Release();
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
                Proto.IGameBackend msg;
                ReadQueueBlocking(out msg);
                // TODO: Need to impl serialize for IServiceMessage...
                listener.Send(msg.Serialize());
            }
        }

    }
}