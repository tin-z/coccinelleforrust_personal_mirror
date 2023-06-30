transformed=$(cargo run --bin main -- ./src/tests/test)
echo $transformed

ans=""
read -p "Is this the expected output(0/1)?" ans
if [ $ans -eq 1 ]
then
    num=$RANDOM
    cp ./src/tests/test.cocci ./src/tests/test$num.cocci
    cp ./src/tests/test.rs ./src/tests/test$num.rs
    echo $transformed > ./src/tests/expected$num.rs
fi