package org.droidless.collections;
import java.util.Arrays;
import java.util.Comparator;

/** The normal contract runs unchanged on desktop Java and in guest DEX. */
public class ArraySortContract {
    static Item[] changing;
    static class Item implements Comparable<Item> {
        int value, order;
        Item(int value,int order) { this.value=value; this.order=order; }
        public int compareTo(Item other) { System.gc(); return value-other.value; }
    }
    static void require(boolean condition) { if (!condition) throw new IllegalStateException("array sort contract"); }
    public static int contract() {
        Item[] values = {new Item(2,0),new Item(1,1),new Item(1,2)};
        Arrays.sort(values);
        require(values[0].order==1 && values[1].order==2 && values[2].order==0);
        Arrays.sort(values,new Comparator<Item>() { public int compare(Item a,Item b) { System.gc(); return b.value-a.value; } });
        require(values[0].order==0 && values[1].order==1 && values[2].order==2);
        Item prefix=new Item(99,0),suffix=new Item(88,4);
        values=new Item[]{prefix,new Item(2,1),new Item(1,2),new Item(1,3),suffix};
        Arrays.sort(values,1,4,null);
        require(values[0]==prefix && values[4]==suffix && values[1].order==2 && values[2].order==3 && values[3].order==1);
        Arrays.sort(values,1,1);
        Object[] nullable = {new Item(1,0),null,new Item(0,1)};
        Arrays.sort(nullable,new Comparator<Object>() { public int compare(Object a,Object b) {
            System.gc(); return a==null ? (b==null ? 0 : -1) : b==null ? 1 : ((Item)a).value-((Item)b).value;
        }});
        require(nullable[0]==null && ((Item)nullable[1]).value==0 && ((Item)nullable[2]).value==1);
        String[] strings={"b","a","a"}; Arrays.sort(strings,null); require("a".equals(strings[0]) && "b".equals(strings[2]));
        int faults=0;
        try { Arrays.sort((Object[])null); } catch (NullPointerException expected) {faults++;}
        try { Arrays.sort(strings,2,1); } catch (IllegalArgumentException expected) {faults++;}
        try { Arrays.sort(strings,-1,1); } catch (ArrayIndexOutOfBoundsException expected) {faults++;}
        try { Arrays.sort(strings,0,4); } catch (ArrayIndexOutOfBoundsException expected) {faults++;}
        try { Arrays.sort(new Object[]{new Object(),new Object()}); } catch (ClassCastException expected) {faults++;}
        Item[] failing={new Item(2,0),new Item(1,1)};
        try { Arrays.sort(failing,new Comparator<Item>() { public int compare(Item a,Item b) { System.gc(); throw new IllegalStateException("comparison failed"); } }); }
        catch (IllegalStateException expected) { faults++; }
        require(faults==6); System.gc(); return 1;
    }
    public static Object prepareMutation() { changing=new Item[]{new Item(2,0),new Item(1,1)}; return changing[0]; }
    public static void mutate() {
        Arrays.sort(changing,new Comparator<Item>() { public int compare(Item a,Item b) { changing[0]=new Item(99,9); System.gc(); return a.value-b.value; } });
    }
    public static int mutationKept() { return changing[0].value==99 ? 1 : 0; }
    public static void capacity() { Arrays.sort(new Object[16385],new Comparator<Object>() { public int compare(Object a,Object b) { return 0; } }); }
    public static void main(String[] args) { require(contract()==1); System.out.println("Array sort contract passed"); }
}
