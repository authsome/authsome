import { Address, Provider, TestUtils, Wallet } from 'fuels';

// Import the contract factory from the folder generated by the fuelchain
// command
import { ContractsAbi__factory } from './contracts';

import LoginForm from './LoginForm';
import WalletView from './WalletView';

// The private key of the `owner` in chainConfig.json.
// This enables us to have an account with an initial balance.
const WALLET_SECRET =
    '0xa449b1ffee0e2205fa924c6740cc48b3b473aa28587df6dab12abc245d1f5298';

// The ID of the contract deployed to our local node.
// The contract ID is displayed when the `forc deploy` command is run.
// E.g. Contract id: 0xa326e3472fd4abc417ba43e369f59ea44f8325d42ba6cf71ec4b58123fd8668a
// const CONTRACT_ID = "0xa326e3472fd4abc417ba43e369f59ea44f8325d42ba6cf71ec4b58123fd8668a"
const CONTRACT_ID = '0xd97ac31a1a473e6dd6f7dfdaf773a2a4df452ec3445afd291298b48dcea57cd3';

// Create a "Wallet" using the private key above.
const wallet = new Wallet(WALLET_SECRET);

// Connect a "Contract" instance using the ID of the deployed contract and the
// wallet above.
// const contract = ContractsAbi__factory.connect(CONTRACT_ID, wallet);

function App() {
    return (
        <>
            <div className="h-screen w-100 bg-green-100">
                <div className="flex min-h-full flex-col justify-center py-12 sm:px-6 lg:px-8">
                    <div className="sm:mx-auto sm:w-full sm:max-w-md">
                        {/* <img
                            className="mx-auto h-12 w-auto"
                            src="https://tailwindui.com/img/logos/mark.svg?color=emerald&shade=600"
                            alt="Your Company"
                        /> */}
                        <svg
                            xmlns="http://www.w3.org/2000/svg"
                            fill="none"
                            viewBox="0 0 24 24"
                            strokeWidth="1.5"
                            stroke="currentColor"
                            className="w-12 h-12 text-emerald-600 mx-auto"
                        >
                            <path
                                strokeLinecap="round"
                                strokeLinejoin="round"
                                d="M15.75 5.25a3 3 0 013 3m3 0a6 6 0 01-7.029 5.912c-.563-.097-1.159.026-1.563.43L10.5 17.25H8.25v2.25H6v2.25H2.25v-2.818c0-.597.237-1.17.659-1.591l6.499-6.499c.404-.404.527-1 .43-1.563A6 6 0 1121.75 8.25z"
                            />
                        </svg>

                        <h2 className="mt-2 text-center text-4xl font-bold tracking-tight text-gray-900 text-emerald-700">
                            Authsome
                        </h2>
                        <p className="text-center text-lg text-emerald-700">
                            Create your multi-sig wallet.
                        </p>
                    </div>
                    <LoginForm />
                </div>
            </div>
            <div className="h-screen w-100 bg-green-100">
                <div className="flex min-h-full flex-col justify-center py-12 sm:px-6 lg:px-8">
                    <div className="sm:mx-auto sm:w-full sm:max-w-md">
                        {/* <img
                            className="mx-auto h-12 w-auto"
                            src="https://tailwindui.com/img/logos/mark.svg?color=emerald&shade=600"
                            alt="Your Company"
                        /> */}
                        <svg
                            xmlns="http://www.w3.org/2000/svg"
                            fill="none"
                            viewBox="0 0 24 24"
                            strokeWidth="1.5"
                            stroke="currentColor"
                            className="w-12 h-12 text-emerald-600 mx-auto"
                        >
                            <path
                                strokeLinecap="round"
                                strokeLinejoin="round"
                                d="M15.75 5.25a3 3 0 013 3m3 0a6 6 0 01-7.029 5.912c-.563-.097-1.159.026-1.563.43L10.5 17.25H8.25v2.25H6v2.25H2.25v-2.818c0-.597.237-1.17.659-1.591l6.499-6.499c.404-.404.527-1 .43-1.563A6 6 0 1121.75 8.25z"
                            />
                        </svg>

                        <h2 className="mt-2 text-center text-4xl font-bold tracking-tight text-gray-900 text-emerald-700">
                            Wallet
                        </h2>
                        <p className="text-center text-lg text-emerald-700">
                            (Logged in screen?)
                        </p>
                    </div>
                    <WalletView />
                </div>
            </div>
        </>
    );
}

export default App;
