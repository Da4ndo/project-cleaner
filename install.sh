#!/bin/sh

pre_setup() {
    # Create .da4ndo directory
    if [ ! -d ~/.da4ndo ]; then
        echo -ne "\033[0;35mCreating\033[0m ~/.da4ndo directory...\033[0m\r"
        mkdir -p ~/.da4ndo
        echo -e "\033[0;32mCreated\033[0m ~/.da4ndo directory.     \033[0m"
    fi

    # Check if env file is already there
    if [ ! -f ~/.da4ndo/env ]; then
        # Download env file
        echo -ne "\033[0;35mDownloading\033[0m env file...\033[0m\r"
        if ! wget https://cdn.da4ndo.com/env; then
            if ! curl -O https://cdn.da4ndo.com/env; then
                echo -e "\033[0;31mFailed to download the env file. Please check your internet connection and whether wget or curl is installed, then try again.\033[0m"
                exit 1
            fi
        fi
        echo -e "\033[0;32mDownloaded\033[0m env file.      \033[0m"

        # Move the env file to the new directory
        echo -ne "\033[0;35mMoving\033[0m env file to ~/.da4ndo/...\033[0m\r"
        if ! mv env ~/.da4ndo/; then
            echo -e "\033[0;31mFailed to move the env file to ~/.da4ndo/. Please check your permissions and try again.\033[0m"
            exit 1
        fi
        echo -e "\033[0;32mMoved\033[0m env file to ~/.da4ndo/.    \033[0m"
    fi

    # Check if ~/.da4ndo/env is in PATH
    if [[ ":$PATH:" != *":$HOME/.da4ndo/bin:"* ]] || { ! grep -q '. "$HOME/.da4ndo/env"' ~/.bashrc && ! grep -q '. "$HOME/.da4ndo/env"' ~/.zshrc; }; then
        # Check if .bashrc exists
        if [ -f ~/.bashrc ]; then
            echo -ne "\033[0;35mAdding\033[0m '. "\$HOME/.da4ndo/env"' to ~/.bashrc...\033[0m\r"

            echo -e '\n. "$HOME/.da4ndo/env"' >> ~/.bashrc

            echo -e "\033[0;32mAdded\033[0m '. "\$HOME/.da4ndo/env"' to ~/.bashrc.     \033[0m"
        fi
        # Check if .zshrc exists
        if [ -f ~/.zshrc ]; then
            echo -ne "\033[0;35mAdding\033[0m '. "\$HOME/.da4ndo/env"' to ~/.zshrc...\033[0m\r"

            echo -e '\n. "$HOME/.da4ndo/env"' >>~/.zshrc

            echo -e "\033[0;32mAdded\033[0m '. "\$HOME/.da4ndo/env" to ~/.zshrc.        \033[0m"
        fi
        echo -e "\033[0;32mAdded\033[0m \$HOME/.da4ndo/bin to PATH.     \033[0m"
    fi
}

clean_previous_installations() {
    # Check if project-cleaner is already installed
    if [ -f ~/.da4ndo/bin/project-cleaner ]; then
        echo -e "\033[0;35mRemoving previous installation of project-cleaner...\033[0m"
        rm ~/.da4ndo/bin/project-cleaner -r
        echo -e "\033[0;32mRemoved previous installation of project-cleaner.\033[0m"
    fi
}

setup() {
    # Check if binary is in the current folder or under target/debug or target/release
    if [ -f ./project-cleaner ]; then
        BINARY_PATH="./project-cleaner"
    elif [ -f ./target/release/project-cleaner ]; then
        BINARY_PATH="./target/release/project-cleaner"
    elif [ -f ./target/debug/project-cleaner ]; then
        BINARY_PATH="./target/debug/project-cleaner"
    else
        echo -e "\033[0;31mBinary not found. Please build the project and try again.\033[0m"
        exit 1
    fi

    # Make the binary executable
    echo -ne "\033[0;35mMaking\033[0m the project-cleaner binary executable...\033[0m\r"
    if ! chmod +x $BINARY_PATH; then
        echo -e "\033[0;31mFailed to make the project-cleaner binary executable. Please check your permissions and try again.\033[0m"
        exit 1
    fi
    echo -e "\033[0;32mMade\033[0m the project-cleaner binary executable.     \033[0m"

    # Check if project-cleaner is already installed
    if
        command -v project-cleaner >/dev/null
    then
        echo -e "\033[0;35mUpdating project-cleaner...\033[0m"
        clean_previous_installations
        if ! mv $BINARY_PATH ~/.da4ndo/bin/project-cleaner; then
            echo -e "\033[0;31mFailed to update project-cleaner. Please check your permissions and try again.\033[0m"
            exit 1
        fi
        echo -e "\033[0;32mUpdated project-cleaner.\033[0m"
    else
        echo -e "\033[0;35mproject-cleaner is not installed.\033[0m"
        mkdir -p ~/.da4ndo/bin/
        if ! mv $BINARY_PATH ~/.da4ndo/bin/project-cleaner; then
            echo -e "\033[0;31mFailed to install project-cleaner. Please check your permissions and try again.\033[0m"
            exit 1
        fi
        echo -e "\033[0;32mInstalled project-cleaner.\033[0m"
    fi
}

pre_setup
setup

echo -e "\033[0;32mSetup completed successfully.\033[0m"
if [[ ":$PATH:" != *":$HOME/.da4ndo/bin:"* ]]; then
    echo -e "\033[0;35mPlease source the environment file or open a new terminal tab for the changes to take effect.\033[0m"
    echo -e "\033[0;35m >\033[0m source \$HOME/.da4ndo/env \n"
fi
