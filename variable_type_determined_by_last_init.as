package
{
    public class Test
    {
        public function test():void
        {
            // When Flex/ASC compiles a function, the variable type is determined by the last initialization of the said variable.
            var val = 1.0;
            var val:int = 100;
            var val:String = "adasdas";
        }
    }
}