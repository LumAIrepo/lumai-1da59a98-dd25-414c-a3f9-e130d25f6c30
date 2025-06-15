// pages/index.tsx
import { FC, useState } from 'react';
import { useWallet } from '@solana/wallet-adapter-react';
import { WalletMultiButton } from '@solana/wallet-adapter-react-ui';
import { CreateLaunchForm } from '../components/CreateLaunchForm';
import { ActiveLaunches } from '../components/ActiveLaunches';

const Home: FC = () => {
  const { connected } = useWallet();
  const [activeTab, setActiveTab] = useState<'create' | 'browse'>('browse');

  return (
    <div className="min-h-screen bg-gray-900 text-white">
      <nav className="bg-gray-800 p-4">
        <div className="container mx-auto flex justify-between items-center">
          <h1 className="text-2xl font-bold">Meme Token Launcher</h1>
          <WalletMultiButton className="!bg-purple-600 hover:!bg-purple-700" />
        </div>
      </nav>

      <main className="container mx-auto px-4 py-8">
        <div className="flex space-x-4 mb-8">
          <button
            className={`px-6 py-2 rounded-lg ${
              activeTab === 'browse'
                ? 'bg-purple-600'
                : 'bg-gray-700 hover:bg-gray-600'
            }`}
            onClick={() => setActiveTab('browse')}
          >
            Browse Launches
          </button>
          <button
            className={`px-6 py-2 rounded-lg ${
              activeTab === 'create'
                ? 'bg-purple-600'
                : 'bg-gray-700 hover:bg-gray-600'
            }`}
            onClick={() => setActiveTab('create')}
          >
            Create Launch
          </button>
        </div>

        {connected ? (
          activeTab === 'create' ? (
            <CreateLaunchForm />
          ) : (
            <ActiveLaunches />
          )
        ) : (
          <div className="text-center py-12">
            <h2 className="text-xl mb-4">Connect your wallet to continue</h2>
          </div>
        )}
      </main>
    </div>
  );
};

export default Home;