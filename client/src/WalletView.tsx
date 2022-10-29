import { useState, useEffect } from 'react';
import { useForm } from 'react-hook-form';

export default function WalletView() {
    const [loginError, setLoginError] = useState('');
    const {
        register,
        handleSubmit,
        watch,
        // formState: { errors },
    } = useForm();

    // Reset register errors
    const watchEmail = watch('email');
    useEffect(() => {
        if (loginError) {
            setLoginError('');
        }
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [watchEmail]);

    // Submit logic here
    const onSubmit = (data: any) => {
        console.log('data', data);
        if (!data.email.includes('@')) {
            setLoginError('Please enter a valid email.');
            return;
        }
        if (data.email === '') {
            setLoginError('Please enter a valid email.');
            return;
        }
    };
    return (
        <>
            <div className="mt-8 sm:mx-auto sm:w-full sm:max-w-md">
                <div className="bg-white py-8 px-4 shadow sm:rounded-lg sm:px-10">
                    <div className="flex items-center">
                        <svg
                            xmlns="http://www.w3.org/2000/svg"
                            fill="none"
                            viewBox="0 0 24 24"
                            strokeWidth={1.5}
                            stroke="currentColor"
                            className="w-5 h-5 mr-2 text-gray-500"
                        >
                            <path
                                strokeLinecap="round"
                                strokeLinejoin="round"
                                d="M21 12a2.25 2.25 0 00-2.25-2.25H15a3 3 0 11-6 0H5.25A2.25 2.25 0 003 12m18 0v6a2.25 2.25 0 01-2.25 2.25H5.25A2.25 2.25 0 013 18v-6m18 0V9M3 12V9m18 0a2.25 2.25 0 00-2.25-2.25H5.25A2.25 2.25 0 003 9m18 0V6a2.25 2.25 0 00-2.25-2.25H5.25A2.25 2.25 0 003 6v3"
                            />
                        </svg>
                        <h3 className="text-lg font-medium leading-6  text-gray-500">
                            0x000...
                        </h3>
                    </div>
                    <ul>
                        <li>asset 1</li>
                        <li>asset 2</li>
                        <li>asset 3</li>
                    </ul>
                    <form className="space-y-6" onSubmit={handleSubmit(onSubmit)}>
                        <div className="mt-6">
                            <div className="relative">
                                <div className="absolute inset-0 flex items-center">
                                    <div className="w-full border-t border-gray-300" />
                                </div>
                                <div className="relative flex justify-center text-sm">
                                    <span className="bg-white px-2 text-gray-500">
                                        Transact
                                    </span>
                                </div>
                            </div>

                            <div className="mt-6 grid grid-cols-2 gap-3">
                                <button
                                    onClick={() => console.log('clicked')}
                                    className="inline-flex w-full justify-center items-center rounded-md border border-gray-300 bg-white py-2 px-4 text-sm font-medium text-gray-500 shadow-sm hover:bg-gray-50"
                                >
                                    <svg
                                        xmlns="http://www.w3.org/2000/svg"
                                        fill="none"
                                        viewBox="0 0 24 24"
                                        strokeWidth={3}
                                        stroke="currentColor"
                                        className="w-3.5 h-3.5 mr-1"
                                    >
                                        <path
                                            strokeLinecap="round"
                                            strokeLinejoin="round"
                                            d="M4.5 19.5l15-15m0 0H8.25m11.25 0v11.25"
                                        />
                                    </svg>
                                    SEND
                                </button>

                                <button
                                    onClick={() => console.log('clicked')}
                                    className="inline-flex w-full justify-center items-center rounded-md border border-gray-300 bg-white py-2 px-4 text-sm font-medium text-gray-500 shadow-sm hover:bg-gray-50"
                                >
                                    <svg
                                        xmlns="http://www.w3.org/2000/svg"
                                        fill="none"
                                        viewBox="0 0 24 24"
                                        strokeWidth={3}
                                        stroke="currentColor"
                                        className="w-3.5 h-3.5  mr-1"
                                    >
                                        <path
                                            strokeLinecap="round"
                                            strokeLinejoin="round"
                                            d="M3 16.5v2.25A2.25 2.25 0 005.25 21h13.5A2.25 2.25 0 0021 18.75V16.5M16.5 12L12 16.5m0 0L7.5 12m4.5 4.5V3"
                                        />
                                    </svg>

                                    <span>RECEIVE</span>
                                </button>
                            </div>
                        </div>
                    </form>
                </div>
            </div>
        </>
    );
}